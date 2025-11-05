use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::HasDisplayHandle,
    window::{Window, WindowId},
};

use vislum_render::context::RenderContext;
use vislum_render::graph::pass::FrameGraphSubmitInfo;
use vislum_render::resource::{
    mesh::Vertex,
    pool::ResourceId,
    texture::{Texture, TextureCreateInfo, TextureDimensions, TextureFormat},
};
use vislum_render_rhi::{
    VkHandle, command::{AccessFlags2, CommandPool, ImageLayout, ImageMemoryBarrier2, IndexType, PipelineBindPoint, PipelineStageFlags2, Rect2D, Viewport}, device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures}, image::Extent2D, instance::{Instance, InstanceExtensions, Library}, memory::MemoryAllocator, queue::Queue, surface::Surface, swapchain::{Swapchain, SwapchainCreateInfo}, sync::{Fence, Semaphore}
};
use vislum_shader::compiler::ShaderCompiler;

#[derive(Default)]
enum AppState {
    #[default]
    Uninitialized,
    Ready {
        window: Arc<Window>,
        render_context: RenderContext,
        surface: Arc<Surface>,
        swapchain: Arc<Swapchain>,
        swapchain_images: Vec<Arc<vislum_render_rhi::image::Image>>,
        texture_id: ResourceId<Texture>,
        // Direct ash handles for things not yet in RHI
        device: Arc<vislum_render_rhi::device::Device>,
        queue: Arc<Queue>,
        // Pipeline and descriptor set (ash directly)
        pipeline_layout: vk::PipelineLayout,
        pipeline: vk::Pipeline,
        descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_set: vk::DescriptorSet,
        descriptor_pool: vk::DescriptorPool,
        sampler: Arc<vislum_render_rhi::sampler::Sampler>,
        image_view: Arc<vislum_render_rhi::image::ImageView>,
        // Mesh (using vislum-render abstraction)
        mesh_id: ResourceId<vislum_render::resource::mesh::Mesh>,
        // Command pool for frame rendering
        command_pool: Arc<CommandPool>,
        // Per-swapchain-image sync objects (one set per swapchain image)
        frame_sync_objects: Vec<(Arc<Semaphore>, Arc<Semaphore>, Arc<Fence>)>, // (acquire, render, fence)
        // Track which swapchain images have been used (to know initial layout)
        swapchain_images_used: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<u32>>>,
        current_frame: usize,
        image_index: Option<u32>,
    },
}

struct App {
    state: AppState,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Uninitialized,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("resumed() called");
        if let AppState::Uninitialized = self.state {
            log::info!("Initializing application...");
            // Create window
            let window_attributes = winit::window::WindowAttributes::default()
                .with_title("Vislum Test - Quad Render")
                .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
                .with_resizable(true)
                .with_visible(true)
                .with_active(true);

            log::info!("Creating window...");
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            log::info!("Window created");

            // Initialize RHI
            log::info!("Creating Vulkan library...");
            let library = Library::new();
            log::info!("Vulkan library created");

            // Build instance extensions
            let mut instance_extensions = InstanceExtensions::default();
            match window.display_handle().unwrap().as_raw() {
                winit::raw_window_handle::RawDisplayHandle::Xlib(_) => {
                    instance_extensions.khr_xlib_surface = true;
                }
                winit::raw_window_handle::RawDisplayHandle::Xcb(_) => {
                    instance_extensions.khr_xcb_surface = true;
                }
                winit::raw_window_handle::RawDisplayHandle::Wayland(_) => {
                    instance_extensions.khr_wayland_surface = true;
                }
                _ => unimplemented!(),
            }
            instance_extensions.khr_surface = true;

            log::info!("Creating Vulkan instance...");
            let instance = Instance::new(library, instance_extensions);
            log::info!("Vulkan instance created");

            log::info!("Creating surface...");
            let surface = Surface::new(instance.clone(), &window);
            log::info!("Surface created");

            // Find suitable physical device
            log::info!("Enumerating physical devices...");
            let physical_devices: Vec<_> = instance.enumerate_physical_devices().collect();
            log::info!("Found {} physical devices", physical_devices.len());
            let (physical_device, queue_family_index) = physical_devices
                .iter()
                .filter_map(|p| {
                    // Check if swapchain extension is supported
                    let extensions = p.extensions();
                    let swapchain_ext_name = ash::khr::swapchain::NAME;
                    if !extensions
                        .iter_c_strs()
                        .any(|name| name == swapchain_ext_name)
                    {
                        return None;
                    }

                    // Check if any queue family supports graphics and presentation
                    let queue_family_index =
                        p.capabilities().enumerate().find_map(|(idx, q)| {
                            if q.queue_flags
                                .contains(vislum_render_rhi::device::QueueFlags::GRAPHICS)
                                && surface.get_physical_device_surface_support(p, idx as u32)
                            {
                                Some(idx)
                            } else {
                                None
                            }
                        })?;

                    Some((p.clone(), queue_family_index as u32))
                })
                .min_by_key(|(p, _)| match p.properties().device_type {
                    vislum_render_rhi::device::PhysicalDeviceType::DISCRETE_GPU => 0,
                    vislum_render_rhi::device::PhysicalDeviceType::INTEGRATED_GPU => 1,
                    vislum_render_rhi::device::PhysicalDeviceType::VIRTUAL_GPU => 2,
                    vislum_render_rhi::device::PhysicalDeviceType::CPU => 3,
                    _ => 4,
                })
                .unwrap();
            log::info!("Selected physical device");

            // Create device
            log::info!("Creating device...");
            let device_ext_names = [ash::khr::swapchain::NAME, ash::khr::dynamic_rendering::NAME];
            let device_extensions = DeviceExtensions::from_iter(device_ext_names.iter().copied());

            // Enable dynamic rendering and synchronization2 features
            let mut device_features = DeviceFeatures::default();
            device_features.dynamic_rendering = true;
            device_features.synchronization2 = true;

            let device = Device::new(
                instance.clone(),
                DeviceCreateInfo {
                    api_version: vislum_render_rhi::Version::V1_3,
                    physical_device,
                    extensions: device_extensions,
                    features: device_features,
                },
            );
            log::info!("Device created");

            // Get queue (device creates queue family 0, so get queue 0)
            log::info!("Getting queue...");
            use vislum_render_rhi::AshHandle;
            let queue_handle =
                unsafe { device.ash_handle().get_device_queue(queue_family_index, 0) };
            let queue = Arc::new(Queue::new(device.clone(), queue_handle));
            log::info!("Queue obtained");

            // Create memory allocator
            log::info!("Creating memory allocator...");
            let _allocator = MemoryAllocator::new(device.clone());
            log::info!("Memory allocator created");

            // Create swapchain
            log::info!("Creating swapchain...");
            let (swapchain, swapchain_images) = Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: None,
                    present_mode: None,
                    image_usage: None,
                    old_swapchain: None,
                },
            );
            log::info!("Swapchain created with {} images", swapchain_images.len());

            // Create RenderContext
            log::info!("Creating render context...");
            let mut render_context = RenderContext::new(device.clone(), queue.clone());
            log::info!("Render context created");

            // Load PNG image
            log::info!("Loading texture image...");
            // Try paths relative to workspace root and package directory
            let image_path =
                PathBuf::from("vislum-test/Rust_programming_language_black_logo.svg.png");
            let img = image::open(&image_path)
                .or_else(|_| image::open("Rust_programming_language_black_logo.svg.png"))
                .expect("Failed to load image: Rust_programming_language_black_logo.svg.png");
            log::info!("Image loaded");
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let image_data = rgba.as_raw();
            log::info!("Image dimensions: {}x{}", width, height);

            // Create texture using vislum-render
            log::info!("Creating texture...");
            let texture_id = render_context.create_texture_with_data(
                TextureCreateInfo {
                    format: TextureFormat::Rgba8Unorm,
                    dimensions: TextureDimensions::D2,
                    extent: vislum_render_rhi::image::Extent3D {
                        width,
                        height,
                        depth: 1,
                    },
                },
                image_data,
            );
            log::info!("Texture created with id: {:?}", texture_id);

            // Create quad mesh using vislum-render
            log::info!("Creating quad mesh...");
            let vertices = vec![
                Vertex {
                    position: [-0.5, -0.5, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    position: [0.5, -0.5, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    uv: [1.0, 0.0],
                },
                Vertex {
                    position: [0.5, 0.5, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    uv: [1.0, 1.0],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    uv: [0.0, 1.0],
                },
            ];

            let indices = vec![0u16, 1, 2, 2, 3, 0];
            let mesh_id = render_context.create_mesh(vertices, indices);
            log::info!("Quad mesh created with id: {:?}", mesh_id);

            // Get texture view for descriptor set
            log::info!("Getting texture view...");
            let image_view = render_context.get_texture_view(texture_id).unwrap();
            log::info!("Texture view obtained");

            // Create sampler
            let sampler = vislum_render_rhi::sampler::Sampler::new(
                device.clone(),
                vislum_render_rhi::sampler::SamplerCreateInfo {
                    mag_filter: vk::Filter::LINEAR,
                    min_filter: vk::Filter::LINEAR,
                    address_mode_u: vk::SamplerAddressMode::REPEAT,
                    address_mode_v: vk::SamplerAddressMode::REPEAT,
                    address_mode_w: vk::SamplerAddressMode::REPEAT,
                },
            );
            log::info!("Image view and sampler created");

            // Compile shaders
            log::info!("Compiling shaders...");
            let compiler = ShaderCompiler::new().expect("Failed to create shader compiler");

            // Try paths relative to workspace root and package directory
            let vert_source = std::fs::read_to_string("vislum-test/shaders/quad.vert.hlsl")
                .or_else(|_| std::fs::read_to_string("shaders/quad.vert.hlsl"))
                .expect("Failed to read vertex shader");
            let frag_source = std::fs::read_to_string("vislum-test/shaders/quad.frag.hlsl")
                .or_else(|_| std::fs::read_to_string("shaders/quad.frag.hlsl"))
                .expect("Failed to read fragment shader");

            let vert_spirv = compiler
                .compile_vertex(&vert_source, "main")
                .expect("Failed to compile vertex shader");
            let frag_spirv = compiler
                .compile_fragment(&frag_source, "main")
                .expect("Failed to compile fragment shader");
            log::info!("Shaders compiled");

            // Create shader modules using ash directly
            log::info!("Creating shader modules...");
            // Convert byte slice to u32 slice for shader code
            let vert_spirv_u32: Vec<u32> = vert_spirv
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            let frag_spirv_u32: Vec<u32> = frag_spirv
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            let vert_shader_module = {
                let create_info = vk::ShaderModuleCreateInfo::default().code(&vert_spirv_u32);
                unsafe {
                    device
                        .ash_handle()
                        .create_shader_module(&create_info, None)
                        .unwrap()
                }
            };

            let frag_shader_module = {
                let create_info = vk::ShaderModuleCreateInfo::default().code(&frag_spirv_u32);
                unsafe {
                    device
                        .ash_handle()
                        .create_shader_module(&create_info, None)
                        .unwrap()
                }
            };
            log::info!("Shader modules created");

            // Create descriptor set layout using ash
            log::info!("Creating descriptor set layout...");
            let descriptor_set_layout = {
                let bindings = [
                    vk::DescriptorSetLayoutBinding::default()
                        .binding(0)
                        .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                        .descriptor_count(1)
                        .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                    vk::DescriptorSetLayoutBinding::default()
                        .binding(1)
                        .descriptor_type(vk::DescriptorType::SAMPLER)
                        .descriptor_count(1)
                        .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                ];

                let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

                unsafe {
                    device
                        .ash_handle()
                        .create_descriptor_set_layout(&create_info, None)
                        .unwrap()
                }
            };

            // Create descriptor pool using ash
            let descriptor_pool = {
                let pool_sizes = [
                    vk::DescriptorPoolSize::default()
                        .ty(vk::DescriptorType::SAMPLED_IMAGE)
                        .descriptor_count(1),
                    vk::DescriptorPoolSize::default()
                        .ty(vk::DescriptorType::SAMPLER)
                        .descriptor_count(1),
                ];

                let create_info = vk::DescriptorPoolCreateInfo::default()
                    .pool_sizes(&pool_sizes)
                    .max_sets(1);

                unsafe {
                    device
                        .ash_handle()
                        .create_descriptor_pool(&create_info, None)
                        .unwrap()
                }
            };

            // Allocate descriptor set using ash
            let descriptor_set = {
                let layouts = [descriptor_set_layout];
                let allocate_info = vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&layouts);

                let sets = unsafe {
                    device
                        .ash_handle()
                        .allocate_descriptor_sets(&allocate_info)
                        .unwrap()
                };
                sets[0]
            };

            // Write descriptor set using ash
            {
                // Binding 0: Sampled image
                use vislum_render_rhi::{VkHandle, command::types::ImageLayout};
                let image_info = vk::DescriptorImageInfo::default()
                    .image_layout(ImageLayout::ShaderReadOnlyOptimal.to_vk())
                    .image_view(image_view.vk_handle());
                let image_infos = [image_info];

                // Binding 1: Sampler
                let sampler_info = vk::DescriptorImageInfo::default().sampler(sampler.vk_handle());
                let sampler_infos = [sampler_info];

                let writes = [
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(0)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                        .image_info(&image_infos),
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(1)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::SAMPLER)
                        .image_info(&sampler_infos),
                ];

                unsafe {
                    device.ash_handle().update_descriptor_sets(&writes, &[]);
                }
            }

            // Create pipeline layout using ash
            let pipeline_layout = {
                let layouts = [descriptor_set_layout];
                let create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);

                unsafe {
                    device
                        .ash_handle()
                        .create_pipeline_layout(&create_info, None)
                        .unwrap()
                }
            };

            // Create graphics pipeline using ash
            let pipeline = {
                // Vertex shader stage
                let vert_stage = vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::VERTEX)
                    .module(vert_shader_module)
                    .name(c"main");

                // Fragment shader stage
                let frag_stage = vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(frag_shader_module)
                    .name(c"main");

                let stages = [vert_stage, frag_stage];

                // Vertex input
                let binding_description = vk::VertexInputBindingDescription::default()
                    .binding(0)
                    .stride(std::mem::size_of::<Vertex>() as u32)
                    .input_rate(vk::VertexInputRate::VERTEX);

                let attribute_descriptions = [
                    vk::VertexInputAttributeDescription::default()
                        .binding(0)
                        .location(0)
                        .format(vk::Format::R32G32B32_SFLOAT)
                        .offset(0),
                    vk::VertexInputAttributeDescription::default()
                        .binding(0)
                        .location(1)
                        .format(vk::Format::R32G32B32_SFLOAT)
                        .offset(12),
                    vk::VertexInputAttributeDescription::default()
                        .binding(0)
                        .location(2)
                        .format(vk::Format::R32G32_SFLOAT)
                        .offset(24),
                ];

                let binding_descriptions = [binding_description];
                let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
                    .vertex_binding_descriptions(&binding_descriptions)
                    .vertex_attribute_descriptions(&attribute_descriptions);

                // Input assembly
                let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                    .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

                // Viewport
                let viewport = vk::Viewport::default()
                    .width(800.0)
                    .height(600.0)
                    .max_depth(1.0);

                use vislum_render_rhi::image::Extent2D;
                let scissor = vk::Rect2D::default().extent(Extent2D::new(800, 600).to_vk());

                let viewports = [viewport];
                let scissors = [scissor];
                let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                    .viewports(&viewports)
                    .scissors(&scissors);

                // Rasterization
                let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
                    .polygon_mode(vk::PolygonMode::FILL)
                    .cull_mode(vk::CullModeFlags::BACK)
                    .front_face(vk::FrontFace::CLOCKWISE)
                    .line_width(1.0);

                // Multisample
                let multisample = vk::PipelineMultisampleStateCreateInfo::default()
                    .rasterization_samples(vk::SampleCountFlags::TYPE_1);

                // Color blend (alpha blending enabled)
                let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                    .color_write_mask(
                        vk::ColorComponentFlags::R
                            | vk::ColorComponentFlags::G
                            | vk::ColorComponentFlags::B
                            | vk::ColorComponentFlags::A,
                    )
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .alpha_blend_op(vk::BlendOp::ADD);

                let color_blend_attachments = [color_blend_attachment];
                let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
                    .logic_op_enable(false)
                    .attachments(&color_blend_attachments);

                // Dynamic rendering (KHR extension)
                let image_format_vk = swapchain.image_format().to_vk();
                let color_attachment_formats = [image_format_vk];
                let mut dynamic_rendering = vk::PipelineRenderingCreateInfo::default()
                    .color_attachment_formats(&color_attachment_formats);

                let create_info = vk::GraphicsPipelineCreateInfo::default()
                    .stages(&stages)
                    .vertex_input_state(&vertex_input)
                    .input_assembly_state(&input_assembly)
                    .viewport_state(&viewport_state)
                    .rasterization_state(&rasterization)
                    .multisample_state(&multisample)
                    .color_blend_state(&color_blend)
                    .layout(pipeline_layout)
                    .push_next(&mut dynamic_rendering);

                let pipelines = unsafe {
                    device
                        .ash_handle()
                        .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                        .unwrap()
                };
                pipelines[0]
            };

            // Create command pool
            log::info!("Creating command pool...");
            let command_pool = CommandPool::new(device.clone(), queue_family_index);
            log::info!("Command pool created");

            // Create per-frame sync objects (one set per swapchain image)
            log::info!("Creating per-frame sync objects...");
            let num_frames = swapchain_images.len();
            log::info!(
                "Creating {} sync object sets for {} swapchain images",
                num_frames,
                num_frames
            );
            let mut frame_sync_objects = Vec::with_capacity(num_frames);
            for _ in 0..num_frames {
                frame_sync_objects.push((
                    Semaphore::new(device.clone()),  // acquire semaphore
                    Semaphore::new(device.clone()),  // render semaphore
                    Fence::signaled(device.clone()), // fence (starts signaled)
                ));
            }

            self.state = AppState::Ready {
                window,
                render_context,
                surface,
                swapchain,
                swapchain_images,
                texture_id,
                device,
                queue,
                pipeline_layout,
                pipeline,
                descriptor_set_layout,
                descriptor_set,
                descriptor_pool,
                sampler,
                image_view,
                mesh_id,
                command_pool,
                frame_sync_objects,
                swapchain_images_used: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
                current_frame: 0,
                image_index: None,
            };
            log::info!("Application state initialized successfully");
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw when about to wait (before first frame)
        if let AppState::Ready { window, .. } = &self.state {
            window.request_redraw();
        } else {
            log::warn!("about_to_wait() called but state is not Ready");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                log::debug!("RedrawRequested event received");
                if let AppState::Ready {
                    render_context,
                    swapchain,
                    swapchain_images,
                    device,
                    queue,
                    pipeline,
                    pipeline_layout,
                    descriptor_set,
                    mesh_id,
                    window,
                    command_pool: _command_pool,
                    frame_sync_objects,
                    swapchain_images_used,
                    current_frame,
                    image_index,
                    texture_id,
                    ..
                } = &mut self.state
                {
                    log::debug!("Processing frame {}", *current_frame);
                    
                    // Get sync objects for current frame
                    let (acquire_semaphore, render_semaphore, render_fence) =
                        &frame_sync_objects[*current_frame];

                    // Wait for fence from previous use of this frame
                    log::debug!("Waiting for fence (status: {})...", render_fence.status());
                    render_fence.wait(u64::MAX);
                    render_fence.reset();
                    log::debug!("Fence reset complete");

                    // Acquire next swapchain image
                    log::debug!("Acquiring swapchain image...");
                    let (img_idx, suboptimal) =
                        swapchain.acquire_next_image(u64::MAX, Some(&acquire_semaphore), None);

                    *image_index = Some(img_idx);
                    log::debug!("Acquired swapchain image {}", img_idx);

                    if suboptimal {
                        log::warn!("Swapchain is suboptimal");
                        // Could recreate swapchain here if needed
                    }

                    // Get swapchain image
                    let swapchain_image = swapchain_images[img_idx as usize].clone();
                    
                    // Check if this image has been used before (to determine initial layout)
                    let is_first_use = {
                        let mut used = swapchain_images_used.lock().unwrap();
                        used.insert(img_idx)
                    };

                    // Create image view for swapchain image using RHI
                    log::debug!("Creating swapchain image view...");
                    use vislum_render_rhi::image::{ImageView, ImageViewCreateInfo, ImageViewType};
                    let swapchain_image_view = ImageView::new(
                        device.clone(),
                        ImageViewCreateInfo {
                            image: swapchain_image.clone(),
                            view_type: ImageViewType::D2,
                            format: swapchain.image_format(),
                            components: vk::ComponentMapping::default(),
                            subresource_range: vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_mip_level(0)
                                .level_count(1)
                                .base_array_layer(0)
                                .layer_count(1),
                        },
                    );
                    log::debug!("Swapchain image view created");

                    let window_size = window.inner_size();
                    let window_width = window_size.width;
                    let window_height = window_size.height;

                    // Set up render pass for this frame using frame graph
                    // Note: We add the pass fresh each frame because frame graph drains nodes
                    log::debug!("Adding render pass for frame...");

                    // Add render pass to frame graph
                    struct RenderQuadNode {
                        swapchain_image: Arc<vislum_render_rhi::image::Image>,
                        swapchain_image_view: Arc<vislum_render_rhi::image::ImageView>,
                        window_width: u32,
                        window_height: u32,
                        pipeline: vk::Pipeline,
                        pipeline_layout: vk::PipelineLayout,
                        descriptor_set: vk::DescriptorSet,
                        mesh_id: vislum_render::resource::pool::ResourceId<
                            vislum_render::resource::mesh::Mesh,
                        >,
                        texture_id: vislum_render::resource::pool::ResourceId<vislum_render::resource::texture::Texture>,
                        is_first_use: bool,
                    }

                    impl vislum_render::graph::FrameNode for RenderQuadNode {
                        fn name(&self) -> std::borrow::Cow<'static, str> {
                            "render_quad".into()
                        }

                        fn prepare(
                            &self,
                            context: &mut vislum_render::graph::PrepareContext,
                        ) -> Box<dyn FnMut(&mut vislum_render::graph::ExecuteContext) + 'static>
                        {
                            let swapchain_image = self.swapchain_image.clone();
                            let swapchain_image_view = self.swapchain_image_view.clone();
                            let window_width = self.window_width;
                            let window_height = self.window_height;
                            let pipeline = self.pipeline;
                            let pipeline_layout = self.pipeline_layout;
                            let descriptor_set = self.descriptor_set;

                            // Read mesh from ResourceManager
                            let mesh = context.read_mesh(self.mesh_id).unwrap();
                            let vertex_buffer = mesh.vertex_buffer();
                            let index_buffer = mesh.index_buffer();
                            
                            // Read texture to ensure it's ready - clone the Arc for the closure
                            let texture_image = context.read_texture(self.texture_id).map(|img| img.clone());
                            let is_first_use = self.is_first_use;

                            Box::new(move |execute_context| {
                                let cmd = &mut execute_context.command_buffer;

                                // Transition swapchain image to the color attachment layout
                                cmd.pipeline_barrier(
                                    std::iter::empty(),
                                    std::iter::empty(),
                                    std::iter::once(ImageMemoryBarrier2 {
                                        image: swapchain_image.clone(),
                                        src_stage_mask: PipelineStageFlags2::TOP_OF_PIPE,
                                        src_access_mask: AccessFlags2::NONE,
                                        dst_stage_mask: PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                                        dst_access_mask: AccessFlags2::COLOR_ATTACHMENT_WRITE,
                                        old_layout: ImageLayout::Undefined,
                                        new_layout: ImageLayout::ColorAttachmentOptimal,
                                    }),
                                );

                                // Begin dynamic rendering
                                let clear_value = vk::ClearValue {
                                    color: vk::ClearColorValue {
                                        float32: [1.0, 1.0, 1.0, 1.0],
                                    },
                                };
                                let color_attachment = vk::RenderingAttachmentInfo::default()
                                    .image_view(swapchain_image_view.vk_handle())
                                    .image_layout(ImageLayout::ColorAttachmentOptimal.to_vk())
                                    .load_op(vk::AttachmentLoadOp::CLEAR)
                                    .store_op(vk::AttachmentStoreOp::STORE)
                                    .clear_value(clear_value);

                                let render_area = vk::Rect2D::default()
                                    .extent(Extent2D::new(window_width, window_height).to_vk());
                                let color_attachments = [color_attachment];
                                let rendering_info = vk::RenderingInfo::default()
                                    .color_attachments(&color_attachments)
                                    // Don't call depth_attachment() or stencil_attachment() to leave them as None
                                    .render_area(render_area)
                                    .layer_count(1);

                                cmd.begin_rendering(&rendering_info);

                                let viewport = Viewport {
                                    x: 0.0,
                                    y: 0.0,
                                    width: window_width as f32,
                                    height: window_height as f32,
                                    min_depth: 0.0,
                                    max_depth: 1.0,
                                };
                                cmd.set_viewport(0, [viewport]);

                                let scissor =
                                    Rect2D::new([0, 0], Extent2D::new(window_width, window_height));
                                cmd.set_scissor(0, [scissor]);

                                // Bind pipeline
                                cmd.bind_pipeline(PipelineBindPoint::Graphics, pipeline);

                                // Bind descriptor set
                                let descriptor_sets = [descriptor_set];
                                cmd.bind_descriptor_sets(
                                    PipelineBindPoint::Graphics,
                                    pipeline_layout,
                                    0,
                                    descriptor_sets,
                                    [],
                                );

                                // Bind vertex buffer
                                let vertex_offsets = [0u64];
                                cmd.bind_vertex_buffers(
                                    0,
                                    [vertex_buffer.clone()],
                                    vertex_offsets,
                                );

                                // Bind index buffer
                                cmd.bind_index_buffer(
                                    index_buffer.clone(),
                                    0,
                                    IndexType::Uint16,
                                );

                                // Draw
                                cmd.draw_indexed(6, 1, 0, 0, 0);

                                // End rendering
                                cmd.end_rendering();

                                // Transition swapchain image to present layout
                                cmd.pipeline_barrier(
                                    std::iter::empty(),
                                    std::iter::empty(),
                                    std::iter::once(ImageMemoryBarrier2 {
                                        image: swapchain_image.clone(),
                                        src_stage_mask: PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                                        src_access_mask: AccessFlags2::COLOR_ATTACHMENT_WRITE,
                                        dst_stage_mask: PipelineStageFlags2::BOTTOM_OF_PIPE,
                                        dst_access_mask: AccessFlags2::NONE,
                                        old_layout: ImageLayout::ColorAttachmentOptimal,
                                        new_layout: ImageLayout::PresentSrcKhr,
                                    }),
                                );
                            })
                        }
                    }

                    let pipeline_copy = *pipeline;
                    let pipeline_layout_copy = *pipeline_layout;
                    let descriptor_set_copy = *descriptor_set;

                    render_context.add_pass(RenderQuadNode {
                        swapchain_image: swapchain_image.clone(),
                        swapchain_image_view,
                        window_width,
                        window_height,
                        pipeline: pipeline_copy,
                        pipeline_layout: pipeline_layout_copy,
                        descriptor_set: descriptor_set_copy,
                        mesh_id: *mesh_id,
                        texture_id: *texture_id,
                        is_first_use,
                    });

                    // Execute render pass
                    log::debug!("Executing and submitting frame graph...");
                    render_context.execute_and_submit(FrameGraphSubmitInfo {
                        wait_semaphores: vec![acquire_semaphore.clone()],
                        signal_semaphores: vec![render_semaphore.clone()],
                        signal_fence: Some(render_fence.clone()),
                    });
                    log::debug!("Frame graph executed and submitted");

                    // Present
                    log::debug!("Presenting swapchain image...");
                    swapchain.present(queue, img_idx, &[&render_semaphore]);

                    // Swapchain image view is automatically cleaned up when dropped (RHI manages it)

                    // Advance to next frame index (for tracking purposes, but we use image index for semaphores)
                    *current_frame = (*current_frame + 1) % frame_sync_objects.len();
                    log::debug!("Frame complete, advanced to frame {}", *current_frame);

                    // Request redraw
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    log::info!("Starting application...");

    let event_loop = EventLoop::new().unwrap();
    log::info!("Event loop created, running application...");
    event_loop.run_app(&mut App::new())?;

    Ok(())
}

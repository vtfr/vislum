use std::sync::Arc;
use std::path::PathBuf;

use anyhow::Result;
use ash::vk;
use image::GenericImageView;
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use vislum_render::context::RenderContext;
use vislum_render::graph::pass::FrameGraphSubmitInfo;
use vislum_render::resource::{pool::ResourceId, texture::{Texture, TextureDimensions, TextureFormat}};
use vislum_render_rhi::{
    instance::{Instance, InstanceExtensions, Library},
    device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, PhysicalDevice},
    queue::Queue,
    surface::Surface,
    swapchain::{Swapchain, SwapchainCreateInfo},
    memory::{MemoryAllocator, MemoryLocation},
    sync::{Fence, Semaphore},
    command::{CommandPool, AutoCommandBuffer},
};
use vislum_shader::compiler::{ShaderCompiler, ShaderType};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

#[derive(Default)]
enum AppState {
    #[default]
    Uninitialized,
    Ready {
        window: Arc<Window>,
        render_context: RenderContext,
        surface: Arc<Surface>,
        swapchain: Arc<Swapchain>,
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
        image_view: Arc<vislum_render_rhi::image_view::ImageView>,
        // Vertex/index buffers (using RHI buffers)
        vertex_buffer: Arc<vislum_render_rhi::buffer::Buffer>,
        index_buffer: Arc<vislum_render_rhi::buffer::Buffer>,
        // Command pool for frame rendering
        command_pool: Arc<CommandPool>,
        // Per-frame sync objects (one set per swapchain image)
        frame_sync_objects: Vec<(Arc<Semaphore>, Arc<Semaphore>, Arc<Fence>)>, // (acquire, render, fence)
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
            let window = Arc::new(
                event_loop.create_window(window_attributes).unwrap()
            );
            log::info!("Window created");

            // Initialize RHI
            log::info!("Creating Vulkan library...");
            let library = Library::new();
            log::info!("Vulkan library created");
            
            // Build instance extensions
            let mut instance_ext_names = vec![ash::khr::surface::NAME];
            #[cfg(unix)]
            {
                instance_ext_names.push(ash::khr::wayland_surface::NAME);
                instance_ext_names.push(ash::khr::xlib_surface::NAME);
                instance_ext_names.push(ash::khr::xcb_surface::NAME);
            }
            #[cfg(windows)]
            {
                instance_ext_names.push(ash::khr::win32_surface::NAME);
            }
            let instance_extensions = InstanceExtensions::from_iter(instance_ext_names.iter().copied());
            
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
                    if !extensions.iter_c_strs().any(|name| name == swapchain_ext_name) {
                        return None;
                    }
                    
                    // Check if any queue family supports graphics and presentation
                    let queue_family_index = p
                        .capabilities()
                            .enumerate()
                            .find_map(|(idx, q)| {
                            if q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
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
                    vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                    vk::PhysicalDeviceType::CPU => 3,
                    _ => 4,
                })
                .unwrap();
            log::info!("Selected physical device");

            // Create device
            log::info!("Creating device...");
            let device_ext_names = [
                ash::khr::swapchain::NAME,
                ash::khr::dynamic_rendering::NAME,
            ];
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
            let queue = Arc::new(device.get_queue(queue_family_index, 0));
            log::info!("Queue obtained");

            // Create memory allocator
            log::info!("Creating memory allocator...");
            let allocator = MemoryAllocator::new(device.clone());
            log::info!("Memory allocator created");

            // Create swapchain
            log::info!("Creating swapchain...");
            let swapchain = Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: None,
                    present_mode: None,
                    image_usage: None,
                    old_swapchain: None,
                },
            );
            log::info!("Swapchain created with {} images", swapchain.images().len());

            // Create RenderContext
            log::info!("Creating render context...");
            let mut render_context = RenderContext::new(
                device.clone(),
                queue.clone(),
                allocator.clone(),
            );
            log::info!("Render context created");

            // Load PNG image
            log::info!("Loading texture image...");
            // Try paths relative to workspace root and package directory
            let image_path = PathBuf::from("vislum-test/Rust_programming_language_black_logo.svg.png");
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
            let texture_id = render_context.create_texture_with_extent(
                TextureFormat::Rgba8Unorm,
                TextureDimensions::D2,
                image_data,
                Some([width, height, 1]),
            );
            log::info!("Texture created with id: {:?}", texture_id);

            // Create quad vertex and index buffers using RHI
            log::info!("Creating vertex and index buffers...");
            let vertices = vec![
                QuadVertex { position: [-0.5, -0.5, 0.0], uv: [0.0, 1.0] },
                QuadVertex { position: [0.5, -0.5, 0.0], uv: [1.0, 1.0] },
                QuadVertex { position: [0.5, 0.5, 0.0], uv: [1.0, 0.0] },
                QuadVertex { position: [-0.5, 0.5, 0.0], uv: [0.0, 0.0] },
            ];

            let indices = vec![0u32, 1, 2, 2, 3, 0];

            // Create staging buffers with host-visible memory for upload
            log::info!("Uploading vertex/index data to GPU...");
            let vertex_data_size = (vertices.len() * std::mem::size_of::<QuadVertex>()) as u64;
            let index_data_size = (indices.len() * std::mem::size_of::<u32>()) as u64;
            
            // Create staging buffers using RHI
            let vertex_staging_buffer = vislum_render_rhi::buffer::Buffer::new_with_location(
                device.clone(),
                allocator.clone(),
                vislum_render_rhi::buffer::BufferCreateInfo {
                    size: vertex_data_size,
                    usage: vk::BufferUsageFlags::TRANSFER_SRC,
                    flags: vk::BufferCreateFlags::empty(),
                },
                MemoryLocation::CpuToGpu,
            );
            
            let index_staging_buffer = vislum_render_rhi::buffer::Buffer::new_with_location(
                device.clone(),
                allocator.clone(),
                vislum_render_rhi::buffer::BufferCreateInfo {
                    size: index_data_size,
                    usage: vk::BufferUsageFlags::TRANSFER_SRC,
                    flags: vk::BufferCreateFlags::empty(),
                },
                MemoryLocation::CpuToGpu,
            );
            
            // Write data to staging buffers
            unsafe {
                vertex_staging_buffer.write(bytemuck::cast_slice(&vertices));
                index_staging_buffer.write(bytemuck::cast_slice(&indices));
            }
            
            // Create GPU buffers
            let vertex_buffer = vislum_render_rhi::buffer::Buffer::new(
                device.clone(),
                allocator.clone(),
                vislum_render_rhi::buffer::BufferCreateInfo {
                    size: vertex_data_size,
                    usage: vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                    flags: vk::BufferCreateFlags::empty(),
                },
            );

            let index_buffer = vislum_render_rhi::buffer::Buffer::new(
                device.clone(),
                allocator.clone(),
                vislum_render_rhi::buffer::BufferCreateInfo {
                    size: index_data_size,
                    usage: vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                    flags: vk::BufferCreateFlags::empty(),
                },
            );
            
            // Upload vertex and index data using AutoCommandBuffer
            {
                let upload_command_pool = CommandPool::new(device.clone(), queue_family_index);
                let mut upload_command_buffer = upload_command_pool.allocate(vk::CommandBufferLevel::PRIMARY);
                upload_command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
                
                let mut auto_command_buffer = AutoCommandBuffer::new(upload_command_buffer);
                
                // Copy staging buffers to GPU buffers using AutoCommandBuffer
                auto_command_buffer.copy_buffer(
                    &vertex_staging_buffer,
                    &vertex_buffer,
                    0,
                    0,
                    vertex_data_size,
                );
                
                auto_command_buffer.copy_buffer(
                    &index_staging_buffer,
                    &index_buffer,
                    0,
                    0,
                    index_data_size,
                );
                
                auto_command_buffer.end();
                
                // Submit upload
                let upload_fence = Fence::unsignaled(device.clone());
                auto_command_buffer.command_buffer().submit(
                    &queue,
                    &[],
                    &[],
                    Some(&upload_fence),
                );
                
                // Wait for upload to complete
                upload_fence.wait(u64::MAX);
                upload_fence.reset();
            }
            log::info!("Vertex/index buffers uploaded");

            // Texture upload will happen via frame graph on first frame
            // No need to execute here - let it accumulate

            // Get texture image for creating image view and descriptor set
            log::info!("Getting texture image...");
            let texture_image = render_context.get_texture_image(texture_id).unwrap();
            log::info!("Texture image obtained");
            
            // Create image view
            let image_view = vislum_render_rhi::image_view::ImageView::new(
                device.clone(),
                vislum_render_rhi::image_view::ImageViewCreateInfo {
                    image: texture_image.image_handle(),
                    view_type: vk::ImageViewType::TYPE_2D,
                    format: vk::Format::R8G8B8A8_UNORM,
                    components: vk::ComponentMapping::default(),
                    subresource_range: vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                },
            );

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
            let vert_spirv_u32: Vec<u32> = vert_spirv.chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            let frag_spirv_u32: Vec<u32> = frag_spirv.chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            let vert_shader_module = {
                let create_info = vk::ShaderModuleCreateInfo::default()
                    .code(&vert_spirv_u32);
                unsafe {
                    device.ash_handle().create_shader_module(&create_info, None).unwrap()
                }
            };

            let frag_shader_module = {
                let create_info = vk::ShaderModuleCreateInfo::default()
                    .code(&frag_spirv_u32);
                unsafe {
                    device.ash_handle().create_shader_module(&create_info, None).unwrap()
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

                let create_info = vk::DescriptorSetLayoutCreateInfo::default()
                    .bindings(&bindings);

                unsafe {
                    device.ash_handle().create_descriptor_set_layout(&create_info, None).unwrap()
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
                    device.ash_handle().create_descriptor_pool(&create_info, None).unwrap()
                }
            };

            // Allocate descriptor set using ash
            let descriptor_set = {
                let layouts = [descriptor_set_layout];
                let allocate_info = vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&layouts);

                let sets = unsafe {
                    device.ash_handle().allocate_descriptor_sets(&allocate_info).unwrap()
                };
                sets[0]
            };

            // Write descriptor set using ash
            {
                // Binding 0: Sampled image
                let image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(image_view.image_view_handle());
                let image_infos = [image_info];
                
                // Binding 1: Sampler
                let sampler_info = vk::DescriptorImageInfo::default()
                    .sampler(sampler.sampler_handle());
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
                let create_info = vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&layouts);

                unsafe {
                    device.ash_handle().create_pipeline_layout(&create_info, None).unwrap()
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
                    .stride(std::mem::size_of::<QuadVertex>() as u32)
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
                        .format(vk::Format::R32G32_SFLOAT)
                        .offset(12),
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

                let scissor = vk::Rect2D::default()
                    .extent(vk::Extent2D { width: 800, height: 600 });

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

                // Color blend
                let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                    .color_write_mask(
                        vk::ColorComponentFlags::R
                        | vk::ColorComponentFlags::G
                        | vk::ColorComponentFlags::B
                        | vk::ColorComponentFlags::A
                    )
                    .blend_enable(false);

                let color_blend_attachments = [color_blend_attachment];
                let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
                    .logic_op_enable(false)
                    .attachments(&color_blend_attachments);

                // Dynamic rendering (KHR extension)
                let image_format = swapchain.image_format();
                let color_attachment_formats = [image_format];
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
                    device.ash_handle().create_graphics_pipelines(
                        vk::PipelineCache::null(),
                        &[create_info],
                None,
                    ).unwrap()
                };
                pipelines[0]
            };

            // Create command pool
            log::info!("Creating command pool...");
            let command_pool = CommandPool::new(device.clone(), queue_family_index);
            log::info!("Command pool created");

            // Create per-frame sync objects (one set per swapchain image)
            log::info!("Creating per-frame sync objects...");
            let swapchain_images = swapchain.images();
            let num_frames = swapchain_images.len();
            log::info!("Creating {} sync object sets for {} swapchain images", num_frames, num_frames);
            let mut frame_sync_objects = Vec::with_capacity(num_frames);
            for _ in 0..num_frames {
                frame_sync_objects.push((
                    Semaphore::new(device.clone()), // acquire semaphore
                    Semaphore::new(device.clone()), // render semaphore
                    Fence::signaled(device.clone()), // fence (starts signaled)
                ));
            }

            self.state = AppState::Ready {
                window,
                render_context,
                surface,
                swapchain,
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
                vertex_buffer,
                index_buffer,
                command_pool,
                frame_sync_objects,
                current_frame: 0,
                image_index: None,
            };
            log::info!("Application state initialized successfully");
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("about_to_wait() called");
        // Request redraw when about to wait (before first frame)
        if let AppState::Ready { window, .. } = &self.state {
            log::info!("About to wait, requesting initial redraw");
            window.request_redraw();
        } else {
            log::warn!("about_to_wait() called but state is not Ready");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
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
                    device,
                    queue,
                    pipeline,
                    pipeline_layout,
                    descriptor_set,
                    vertex_buffer,
                    index_buffer,
                    window,
                    command_pool,
                    frame_sync_objects,
                    current_frame,
                    image_index,
                    ..
                } = &mut self.state {
                    log::debug!("Processing frame {}", *current_frame);
                    // Get sync objects for current frame
                    let (acquire_semaphore, render_semaphore, render_fence) = &frame_sync_objects[*current_frame];
                    
                    // Wait for fence from previous use of this frame
                    // On first use, fence starts signaled, so we wait and reset
                    log::debug!("Waiting for fence (status: {})...", render_fence.status());
                    let waited = render_fence.wait(u64::MAX);
                    log::debug!("Fence wait returned: {}", waited);
                    if waited {
                        log::debug!("Resetting fence...");
                        render_fence.reset();
                        log::debug!("Fence reset complete");
                    }

                    // Acquire next swapchain image
                    log::debug!("Acquiring swapchain image...");
                    let (img_idx, suboptimal) = swapchain.acquire_next_image(
                        u64::MAX,
                        Some(&acquire_semaphore),
                        None,
                    );

                    *image_index = Some(img_idx);
                    log::debug!("Acquired swapchain image {}", img_idx);

                    if suboptimal {
                        log::warn!("Swapchain is suboptimal");
                        // Could recreate swapchain here if needed
                    }

                    // Get swapchain image
                    let swapchain_images = swapchain.images();
                    let swapchain_image = swapchain_images[img_idx as usize];

                    // Create image view for swapchain image (using ash directly)
                    log::debug!("Creating swapchain image view...");
                    let swapchain_image_view = {
                        let create_info = vk::ImageViewCreateInfo::default()
                            .image(swapchain_image)
                            .view_type(vk::ImageViewType::TYPE_2D)
                            .format(swapchain.image_format())
                            .components(vk::ComponentMapping::default())
                            .subresource_range(vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_mip_level(0)
                                .level_count(1)
                                .base_array_layer(0)
                                .layer_count(1));

                        unsafe {
                            device.ash_handle().create_image_view(&create_info, None).unwrap()
                        }
                    };
                    log::debug!("Swapchain image view created");

                    let window_size = window.inner_size();
                    let window_width = window_size.width;
                    let window_height = window_size.height;

                    // Set up render pass for this frame using frame graph
                    // Note: We add the pass fresh each frame because frame graph drains nodes
                    log::debug!("Adding render pass for frame...");
                    let pipeline_copy = *pipeline;
                    let pipeline_layout_copy = *pipeline_layout;
                    let descriptor_set_copy = *descriptor_set;
                    let vertex_buffer_clone = vertex_buffer.clone();
                    let index_buffer_clone = index_buffer.clone();
                    let swapchain_clone = swapchain.clone();
                    let device_clone = device.clone();

                    // Add render pass to frame graph
                    render_context.add_pass(
                        "render_quad",
                        |_prepare_context| {
                            struct RenderState {
                                swapchain_image: vk::Image,
                                swapchain_image_view: vk::ImageView,
                                window_width: u32,
                                window_height: u32,
                            }

                            RenderState {
                                swapchain_image,
                                swapchain_image_view,
                                window_width,
                                window_height,
                            }
                        },
                        move |execute_context, state| {
                            // Create temporary Image wrapper for transition
                            let swapchain_image = vislum_render_rhi::image::Image::from_swapchain_image(
                                swapchain_clone.clone(),
                                state.swapchain_image,
                            );

                            // Transition swapchain image to color attachment layout
                            execute_context.command_encoder.transition_image(
                                &swapchain_image,
                                vk::ImageLayout::UNDEFINED,
                                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                                vk::AccessFlags2::empty(),
                                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                                vk::PipelineStageFlags2::TOP_OF_PIPE,
                                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                            );

                            // Use AutoCommandBuffer for rendering commands
                            let auto_cmd_buf = execute_context.command_encoder.command_buffer();

                            // Begin dynamic rendering
                            let clear_value = vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [0.0, 0.0, 0.0, 1.0],
                                },
                            };
                            let color_attachment = vk::RenderingAttachmentInfo::default()
                                .image_view(state.swapchain_image_view)
                                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                                .load_op(vk::AttachmentLoadOp::CLEAR)
                                .store_op(vk::AttachmentStoreOp::STORE)
                                .clear_value(clear_value);

                            let render_area = vk::Rect2D::default()
                                .extent(vk::Extent2D {
                                    width: state.window_width,
                                    height: state.window_height,
                                });
                            let color_attachments = [color_attachment];
                            let rendering_info = vk::RenderingInfo::default()
                                .color_attachments(&color_attachments)
                                .render_area(render_area)
                                .layer_count(1);

                            auto_cmd_buf.begin_rendering(&rendering_info);
                            
                            // Set viewport
                            let viewport = vk::Viewport::default()
                                .width(state.window_width as f32)
                                .height(state.window_height as f32)
                                .max_depth(1.0);
                            auto_cmd_buf.set_viewport(0, &[viewport]);

                            // Set scissor
                            let scissor = vk::Rect2D::default()
                                .extent(vk::Extent2D {
                                    width: state.window_width,
                                    height: state.window_height,
                                });
                            auto_cmd_buf.set_scissor(0, &[scissor]);
                            
                            // Bind pipeline
                            auto_cmd_buf.bind_pipeline(
                                vk::PipelineBindPoint::GRAPHICS,
                                pipeline_copy,
                            );
                            
                            // Bind descriptor set
                            let descriptor_sets = [descriptor_set_copy];
                            auto_cmd_buf.bind_descriptor_sets(
                                vk::PipelineBindPoint::GRAPHICS,
                                pipeline_layout_copy,
                                0,
                                &descriptor_sets,
                                &[],
                            );
                            
                            // Bind vertex buffer
                            let vertex_offsets = [0u64];
                            auto_cmd_buf.bind_vertex_buffers_buffers(
                                0,
                                &[&vertex_buffer_clone],
                                &vertex_offsets,
                            );
                        
                            // Bind index buffer
                            auto_cmd_buf.bind_index_buffer_buffer(
                                &index_buffer_clone,
                                0,
                                vk::IndexType::UINT32,
                            );
                            
                            // Draw
                            auto_cmd_buf.draw_indexed(6, 1, 0, 0, 0);
                            
                            // End rendering
                            auto_cmd_buf.end_rendering();

                            // Transition swapchain image to present layout
                            execute_context.command_encoder.transition_image(
                                &swapchain_image,
                                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                                vk::ImageLayout::PRESENT_SRC_KHR,
                                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                                vk::AccessFlags2::empty(),
                                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                                vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                            );
                        },
                    );

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
                    swapchain.present(&queue, img_idx, &[&render_semaphore]);

                    // Cleanup swapchain image view
                    unsafe {
                        device.ash_handle().destroy_image_view(swapchain_image_view, None);
                    }

                    // Advance to next frame
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

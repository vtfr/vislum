use std::sync::Arc;

use ash::vk;
use vislum_rhi::{
    AshHandle, VkHandle, buffer::{Buffer, BufferCreateInfo, BufferUsage}, command::{CommandBuffer, pool::{CommandPool, CommandPoolCreateInfo}}, descriptor::{
        DescriptorPool, DescriptorPoolCreateInfo, DescriptorSetLayout,
        DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, layout::DescriptorType,
    }, device::{
        device::{Device, DeviceCreateInfo},
        ffi::{DeviceExtensions, DeviceFeatures},
    }, image::{Extent2D, Extent3D, ImageCreateInfo, ImageDimensions, ImageFormat, ImageView, ImageViewCreateInfo}, instance::{Instance, InstanceExtensions, Library}, memory::allocator::{MemoryAllocator, MemoryLocation}, pipeline::{GraphicsPipeline, GraphicsPipelineCreateInfo, ShaderModule}, queue::Queue, surface::Surface, swapchain::{Swapchain, SwapchainCreateInfo}, sync::{Fence, Semaphore}, version::Version
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

struct RenderState {
    queue: Arc<Queue>,
    _surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    command_pool: Arc<CommandPool>,
    command_buffers: Vec<CommandBuffer>,
    image_available: Vec<Arc<Semaphore>>,
    render_finished: Vec<Arc<Semaphore>>,
    in_flight_fences: Vec<Arc<Fence>>,
    pipeline: Arc<GraphicsPipeline>,
    current_frame: usize,
}

impl RenderState {
    fn render(&mut self) {
        let frame = self.current_frame;
        
        // Wait for previous frame
        self.in_flight_fences[frame].wait(u64::MAX);
        self.in_flight_fences[frame].reset();
        
        // Acquire next image
        let image_index = match self.swapchain.acquire_next_image(self.image_available[frame].vk_handle()) {
            Some(idx) => idx,
            None => return, // Skip this frame
        };
        
        // Record command buffer
        let cmd = &self.command_buffers[frame];
        cmd.reset(false);
        cmd.begin(false);
        
        // Transition image to color attachment
        let barrier = vk::ImageMemoryBarrier::default()
            .image(self.swapchain.images()[image_index as usize].vk_handle())
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            // .subresource_range(vk::ImageSubresourceRange {
            //     aspect_mask: vk::ImageAspectFlags::COLOR,
            //     base_mip_level: 0,
            //     level_count: 1,
            //     base_array_layer: 0,
            //     layer_count: 1,
            // })
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        
        cmd.pipeline_barrier(
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
        
        // Begin rendering with clear color
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.2, 0.3, 1.0], // Nice blue color
            },
        };
        
        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.swapchain.views()[image_index as usize].vk_handle())
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear_value);
        
        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent(),
            })
            .layer_count(1)
            .color_attachments(std::slice::from_ref(&color_attachment));
        
        cmd.begin_rendering(&rendering_info);
        
        // Set viewport and scissor
        let extent = self.swapchain.extent();
        cmd.set_viewport(0, &[vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }]);
        cmd.set_scissor(0, &[vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        }]);
        
        // Draw triangle
        cmd.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, self.pipeline.as_ref());
        cmd.draw(0..3, 0..1);
        
        cmd.end_rendering();
        
        // Transition to present
        let present_barrier = vk::ImageMemoryBarrier::default()
            .image(self.swapchain.images()[image_index as usize].vk_handle())
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::empty());
        
        cmd.pipeline_barrier(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[present_barrier],
        );
        
        cmd.end();
        
        // Submit
        let wait_semaphores = [self.image_available[frame].vk_handle()];
        let signal_semaphores = [self.render_finished[frame].vk_handle()];
        let command_buffers = [cmd.vk_handle()];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        
        self.queue.submit(&submit_info, self.in_flight_fences[frame].vk_handle());
        
        // Present
        self.swapchain.present(self.queue.vk_handle(), image_index, self.render_finished[frame].vk_handle());
        
        // Next frame
        self.current_frame = (self.current_frame + 1) % 2;
    }
}

struct AppData {
    device: Arc<Device>,
    instance: Arc<Instance>,
    queue: Arc<Queue>,
    image_available: Vec<Arc<Semaphore>>,
    render_finished: Vec<Arc<Semaphore>>,
    in_flight_fences: Vec<Arc<Fence>>,
    pipeline: Arc<GraphicsPipeline>,
}

struct App {
    window: Option<Window>,
    render_state: Option<RenderState>,
    data: Option<AppData>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = Window::default_attributes()
                .with_title("Vislum RHI Demo - Press ESC to exit")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
            let window = event_loop.create_window(window_attrs).unwrap();
            println!("✓ Window created: 800x600");
            
            // Create surface and swapchain now that we have a window
            if let Some(data) = &self.data {
                println!("Creating surface...");
                let surface = Surface::new(data.instance.clone(), &window, &window);
                println!("✓ Surface created");
                
                println!("Creating swapchain...");
                let swapchain = Swapchain::new(
                    data.device.clone(),
                    &surface,
                    SwapchainCreateInfo {
                        extent: Extent2D {
                            width: 800,
                            height: 600,
                        },
                        format: ImageFormat::B8G8R8A8Srgb,
                        present_mode: vk::PresentModeKHR::FIFO,
                    },
                ).expect("failed to create swapchain");
                
                println!("✓ Swapchain created with {} images", swapchain.views().len());
                
                // Create command pool and buffers
                let command_pool = Arc::new(CommandPool::new(data.device.clone(), CommandPoolCreateInfo {
                    queue_family_index: 0,
                    transient: false,
                    reset_command_buffer: true,
                }));
                let command_buffers: Vec<_> = command_pool.allocate_command_buffers(2).collect();
                
                self.render_state = Some(RenderState {
                    queue: data.queue.clone(),
                    _surface: surface,
                    swapchain,
                    command_pool,
                    command_buffers,
                    image_available: data.image_available.clone(),
                    render_finished: data.render_finished.clone(),
                    in_flight_fences: data.in_flight_fences.clone(),
                    pipeline: data.pipeline.clone(),
                    current_frame: 0,
                });
                
                println!("✓ Rendering initialized!");
            }
            
            self.window = Some(window);
            
            // Request first redraw
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\nClosing window...");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) {
                    println!("\nESC pressed, exiting...");
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.render();
                }
                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    println!("=== Vislum RHI Demo ===\n");

    // Create instance
    println!("Creating Vulkan instance...");
    let library = Library::new().expect("failed to load vulkan library");
    let instance = Instance::new(
        library,
        InstanceExtensions {
        khr_surface: true,
        khr_wayland_surface: true,
        ..Default::default()
        },
    ).expect("failed to create vulkan instance");

    // Get physical device
    println!("Enumerating physical devices...");
    let physical_devices = instance.enumerate_physical_devices()
        .expect("failed to enumerate physical devices");
    if physical_devices.is_empty() {
        println!("No physical devices found!");
        return;
    }

    let physical_device = &physical_devices[0];
    println!("Selected device: {:?}", physical_device.device_properties());

    // Create logical device
    println!("\nCreating logical device...");
    let device = Device::new(
        instance.clone(),
        physical_device.clone(),
        DeviceCreateInfo {
            api_version: Version::new(1, 3, 0),
            enabled_extensions: DeviceExtensions {
                khr_swapchain: true,
                khr_synchronization2: true,
                khr_dynamic_rendering: true,
                khr_ext_descriptor_indexing: true,
                ..Default::default()
            },
            enabled_features: DeviceFeatures {
                dynamic_rendering: true,
                ..Default::default()
            },
        },
    );
    println!("✓ Device created");

    // Create memory allocator
    println!("\nCreating memory allocator...");
    let allocator = MemoryAllocator::new(device.clone());
    println!("✓ Allocator created");

    // Create vertex buffer
    println!("\nCreating vertex buffer for triangle...");
    let _vertex_buffer = Buffer::new(
        device.clone(),
        allocator.clone(),
        BufferCreateInfo {
            size: (std::mem::size_of::<Vertex>() * 3) as u64,
            usage: BufferUsage::VERTEX_BUFFER,
            location: MemoryLocation::CpuToGpu,
        },
    ).expect("failed to create vertex buffer");
    println!("✓ Vertex buffer created");

    // Create uniform buffer
    println!("\nCreating uniform buffer...");
    let uniform_buffer = Buffer::new(
        device.clone(),
        allocator.clone(),
        BufferCreateInfo {
            size: 64,
            usage: BufferUsage::UNIFORM_BUFFER,
            location: MemoryLocation::CpuToGpu,
        },
    ).expect("failed to create uniform buffer");
    println!("✓ Uniform buffer created");

    // Create image (render target)
    println!("\nCreating render target image...");
    let image = vislum_rhi::image::Image::new(
        device.clone(),
        allocator.clone(),
        ImageCreateInfo {
            dimensions: ImageDimensions::D2,
            extent: Extent3D {
                width: 800,
                height: 600,
                depth: 1,
            },
            format: ImageFormat::R8G8B8A8Srgb,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        },
    );
    println!("✓ Image created: {}x{}", image.extent().width, image.extent().height);

    // Create image view
    println!("\nCreating image view...");
    let _image_view = ImageView::new(
        image.clone(),
        ImageViewCreateInfo {
            dimensions: ImageDimensions::D2,
            format: ImageFormat::R8G8B8A8Srgb,
        },
    );
    println!("✓ Image view created");

    // Create descriptor set layout
    println!("\nCreating descriptor set layout...");
    let descriptor_layout = DescriptorSetLayout::new(
        device.clone(),
        DescriptorSetLayoutCreateInfo {
            bindings: vec![DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: DescriptorType::UniformBuffer,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
            }],
        },
    );
    println!("✓ Descriptor set layout created");

    // Create descriptor pool
    println!("\nCreating descriptor pool...");
    let descriptor_pool = DescriptorPool::new(
        device.clone(),
        DescriptorPoolCreateInfo::default(),
    );
    println!("✓ Descriptor pool created");

    // Allocate descriptor set
    println!("\nAllocating descriptor set...");
    let descriptor_set = descriptor_pool
        .allocate(std::iter::once(descriptor_layout.clone()))
        .next()
        .unwrap();
    println!("✓ Descriptor set allocated");

    // Write to descriptor set
    println!("\nBinding uniform buffer to descriptor set...");
    descriptor_set.write_buffer(0, &uniform_buffer, 0, uniform_buffer.size());
    println!("✓ Uniform buffer bound");

    // Note: Shader creation would fail with dummy data, so we skip it
    println!("\n--- Pipeline Creation (skipped - requires real SPIR-V) ---");
    println!("Would create:");
    println!("  • Vertex shader module");
    println!("  • Fragment shader module");
    println!("  • Graphics pipeline with:");
    println!("    - Vertex layout: position (vec3) + color (vec3)");
    println!("    - Topology: Triangle list");
    println!("    - Color format: R8G8B8A8 sRGB");
    println!("    - Descriptor set: 1 uniform buffer");

    println!("\n=== RHI Initialized Successfully ===");
    println!("Demonstrated:");
    println!("  ✓ Instance creation");
    println!("  ✓ Physical device enumeration");
    println!("  ✓ Logical device creation");
    println!("  ✓ Memory allocator");
    println!("  ✓ Buffer creation (vertex + uniform)");
    println!("  ✓ Image creation with view");
    println!("  ✓ Descriptor sets (layout, pool, allocation, binding)");

    // Create queue
    println!("\nCreating queue...");
    let queue = Queue::new(device.clone(), 0, 0);
    println!("✓ Queue created");
    
    // Create synchronization primitives
    println!("Creating synchronization primitives...");
    let image_available = vec![
        Semaphore::new(device.clone()),
        Semaphore::new(device.clone()),
    ];
    let render_finished = vec![
        Semaphore::new(device.clone()),
        Semaphore::new(device.clone()),
    ];
    let in_flight_fences = vec![
        Fence::new(device.clone(), true),
        Fence::new(device.clone(), true),
    ];
    println!("✓ Synchronization primitives created");
    
    // Load shaders and create pipeline
    println!("\nLoading shaders...");
    let vert_code = std::fs::read("vislum-test/shaders/triangle_simple.vert.spv").expect("Failed to read vertex shader");
    let frag_code = std::fs::read("vislum-test/shaders/triangle_simple.frag.spv").expect("Failed to read fragment shader");
    
    let vertex_shader = ShaderModule::new(device.clone(), &vert_code);
    let fragment_shader = ShaderModule::new(device.clone(), &frag_code);
    println!("✓ Shaders loaded");
    
    println!("Creating graphics pipeline...");
    let pipeline = GraphicsPipeline::new(
        device.clone(),
        GraphicsPipelineCreateInfo {
            vertex_shader,
            fragment_shader,
            vertex_buffer: None, // Triangle vertices are hardcoded in shader
            descriptor_set_layouts: vec![],
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            color_formats: vec![ImageFormat::B8G8R8A8Srgb],
            depth_format: None,
        },
    );
    println!("✓ Graphics pipeline created");
    
    println!("\n=== Opening Window ===");
    let event_loop = EventLoop::new().unwrap();
    
    let mut app = App {
        window: None,
        render_state: None,
        data: Some(AppData {
            device: device.clone(),
            instance: instance.clone(),
            queue,
            image_available,
            render_finished,
            in_flight_fences,
            pipeline,
        }),
    };
    
    println!("Starting event loop...");
    println!("Press ESC or close window to exit\n");
    
    event_loop.run_app(&mut app).unwrap();
    
    // Wait for device to finish
    println!("\nWaiting for device to finish...");
    unsafe {
        device.ash_handle().device_wait_idle().unwrap();
    }
    
    println!("\n=== Demo Complete ===");
}

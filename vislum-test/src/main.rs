use log::LevelFilter;
use std::sync::Arc;
use vislum_render::rhi::{
    ash::{self, vk},
    command::{CommandBuffer, CommandPool},
    device::{Device, DeviceDescription, DeviceExtensions, DeviceFeatures},
    image::ImageView,
    instance::{Instance, InstanceDescription, InstanceExtensions},
    queue::{Queue, QueueDescription, SubmissionInfo},
    surface::Surface,
    swapchain::{Swapchain, SwapchainDescription},
    sync::Semaphore,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct TriangleDemo {
    instance: Option<Arc<Instance>>,
    window: Option<Arc<Window>>,
    surface: Option<Arc<Surface>>,
    device: Option<Arc<Device>>,
    swapchain: Option<Arc<Swapchain>>,
    pipeline: Option<ash::vk::Pipeline>,
    pipeline_layout: Option<ash::vk::PipelineLayout>,
    image_views: Vec<Arc<ImageView>>,
    command_pool: Option<Arc<CommandPool>>,
    command_buffers: Vec<Arc<CommandBuffer>>,
    image_available_semaphore: Option<Arc<Semaphore>>,
    render_finished_semaphore: Option<Arc<Semaphore>>,
    vertex_shader_module: Option<ash::vk::ShaderModule>,
    fragment_shader_module: Option<ash::vk::ShaderModule>,
    queue: Option<Queue>,
}

impl TriangleDemo {
    fn new() -> Self {
        // Create instance early
        let instance = Instance::new(InstanceDescription {
            extensions: InstanceExtensions {
                khr_surface: true,
                khr_wayland_surface: true,
                ext_debug_utils: true,
                khr_get_physical_device_properties2: true,
                khr_get_surface_capabilities2: true,
                ext_swapchain_colorspace: true,
                khr_portability_enumeration: true,
                ..Default::default()
            },
        })
        .expect("Failed to create Vulkan instance");

        Self {
            instance: Some(instance),
            window: None,
            surface: None,
            device: None,
            swapchain: None,
            pipeline: None,
            pipeline_layout: None,
            image_views: Vec::new(),
            command_pool: None,
            command_buffers: Vec::new(),
            image_available_semaphore: None,
            render_finished_semaphore: None,
            vertex_shader_module: None,
            fragment_shader_module: None,
            queue: None,
        }
    }

    fn init_vulkan(&mut self) {
        let instance = self.instance.as_ref().unwrap();
        let physical_devices = instance.enumerate_physical_devices();

        log::info!("Found {} physical device(s)", physical_devices.len());
        for (i, physical_device) in physical_devices.iter().enumerate() {
            log::info!(
                "Physical device {i}: {}",
                physical_device.supported_extensions()
            );
        }

        // Create device
        let physical_device = physical_devices[0].clone();
        let device = Device::new(DeviceDescription {
            physical_device: physical_device.clone(),
            features: DeviceFeatures {
                dynamic_rendering: true,
                synchronization2: true,
                extended_dynamic_state: true,
            },
            extensions: DeviceExtensions {
                khr_swapchain: true,
                khr_synchronization2: true,
                khr_dynamic_rendering: true,
                ext_extended_dynamic_state: true,
                ..Default::default()
            },
        })
        .expect("Failed to create device");

        log::info!("Device created successfully");
        self.device = Some(device);

        // Create queue
        let queue = Queue::new(
            self.device.as_ref().unwrap().clone(),
            QueueDescription {
                queue_family_index: 0,
                queue_index: 0,
                max_in_flight_submissions: 2,
            },
        );
        self.queue = Some(queue);

        // Create surface
        let window = self.window.as_ref().unwrap();
        let surface = Surface::new(instance.clone(), window).expect("Failed to create surface");
        self.surface = Some(Arc::new(surface));

        // Create swapchain
        let window_size = window.inner_size();
        let swapchain = Swapchain::new(
            self.device.as_ref().unwrap().clone(),
            SwapchainDescription {
                surface: self.surface.as_ref().unwrap().clone(),
                format: Some(ash::vk::Format::B8G8R8A8_UNORM),
                color_space: None,
                present_mode: None,
                image_count: None,
                width: window_size.width,
                height: window_size.height,
            },
        )
        .expect("Failed to create swapchain");

        log::info!(
            "Swapchain created: format={:?}, extent={:?}",
            swapchain.format(),
            swapchain.extent()
        );
        self.swapchain = Some(Arc::new(swapchain));

        // Create shaders
        self.create_shaders();

        // Create pipeline
        self.create_pipeline();

        // Create image views
        self.create_image_views();

        // Create command pool and buffers
        self.create_command_pool();
        self.create_command_buffers();

        // Create synchronization objects
        self.create_sync_objects();
    }

    fn create_shaders(&mut self) {
        // Vertex shader SPIR-V (simple triangle with hardcoded positions)
        let vertex_shader_code = include_bytes!("../shaders/triangle.vert.spv");
        let fragment_shader_code = include_bytes!("../shaders/triangle.frag.spv");

        let vertex_module = self.create_shader_module(vertex_shader_code);
        let fragment_module = self.create_shader_module(fragment_shader_code);

        self.vertex_shader_module = Some(vertex_module);
        self.fragment_shader_module = Some(fragment_module);
    }

    fn create_shader_module(&self, code: &[u8]) -> ash::vk::ShaderModule {
        let device = self.device.as_ref().unwrap();

        // Convert bytes to u32 words
        let code_u32: Vec<u32> = code
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let create_info = vk::ShaderModuleCreateInfo::default().code(&code_u32);

        unsafe {
            device
                .handle()
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        }
    }

    fn create_pipeline(&mut self) {
        let device = self.device.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();

        // Create pipeline layout
        let layout_create_info = vk::PipelineLayoutCreateInfo::default();

        let pipeline_layout = unsafe {
            device
                .handle()
                .create_pipeline_layout(&layout_create_info, None)
                .expect("Failed to create pipeline layout")
        };
        self.pipeline_layout = Some(pipeline_layout);

        // Shader stages
        let entry_point = std::ffi::CString::new("main").unwrap();

        let vert_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(self.vertex_shader_module.unwrap())
            .name(&entry_point);

        let frag_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(self.fragment_shader_module.unwrap())
            .name(&entry_point);

        let shader_stages = [vert_stage, frag_stage];

        // Vertex input (empty - hardcoded in shader)
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

        // Input assembly
        let input_assembly =
            vk::PipelineInputAssemblyStateCreateInfo::default().primitive_restart_enable(false);

        // Viewport state - using dynamic state (count will be set at draw time)
        let viewport_state = vk::PipelineViewportStateCreateInfo::default();

        // Rasterizer - cull mode, front face, and topology are dynamic
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .depth_bias_enable(false);

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Color blending
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blend_attachments = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        // Dynamic state - these will be set in the render loop
        let dynamic_states = [
            vk::DynamicState::VIEWPORT_WITH_COUNT,
            vk::DynamicState::SCISSOR_WITH_COUNT,
            vk::DynamicState::CULL_MODE,
            vk::DynamicState::FRONT_FACE,
            vk::DynamicState::PRIMITIVE_TOPOLOGY,
        ];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        // Dynamic rendering info
        let color_attachment_formats = [swapchain.format()];
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_formats);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .push_next(&mut pipeline_rendering_info);

        let pipeline_infos = [pipeline_info];

        let pipelines = unsafe {
            device
                .handle()
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .expect("Failed to create graphics pipeline")
        };

        self.pipeline = Some(pipelines[0]);
    }

    fn create_image_views(&mut self) {
        let device = self.device.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();

        // Create image views for swapchain images
        for image in swapchain.images() {
            let image_view = ImageView::new(
                device.clone(),
                *image,
                swapchain.format(),
                ash::vk::ImageAspectFlags::COLOR,
            );
            self.image_views.push(Arc::new(image_view));
        }
    }

    fn create_command_pool(&mut self) {
        let device = self.device.as_ref().unwrap();

        let command_pool = CommandPool::new(
            device.clone(),
            0,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )
        .expect("Failed to create command pool");

        self.command_pool = Some(command_pool);
    }

    fn create_command_buffers(&mut self) {
        let command_pool = self.command_pool.as_ref().unwrap();

        let command_buffers = command_pool
            .allocate_command_buffers(
                vk::CommandBufferLevel::PRIMARY,
                self.image_views.len() as u32,
            )
            .expect("Failed to allocate command buffers");

        self.command_buffers = command_buffers.into_iter().map(Arc::new).collect();
    }

    fn create_sync_objects(&mut self) {
        let device = self.device.as_ref().unwrap();

        self.image_available_semaphore = Some(Arc::new(Semaphore::new(device.clone())));
        self.render_finished_semaphore = Some(Arc::new(Semaphore::new(device.clone())));
    }

    fn render_frame(&mut self) {
        let _device = self.device.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();
        let queue = self.queue.as_mut().unwrap();
        let image_available = self.image_available_semaphore.as_ref().unwrap();
        let render_finished = self.render_finished_semaphore.as_ref().unwrap();

        // Acquire next image
        let (image_index, _) = swapchain
            .acquire_next_image(u64::MAX, Some(image_available.handle()), None)
            .expect("Failed to acquire next image");

        // Record command buffer
        let command_buffer = &self.command_buffers[image_index as usize];

        command_buffer.reset(vk::CommandBufferResetFlags::empty());

        command_buffer.begin(vk::CommandBufferUsageFlags::empty());

        // Dynamic rendering
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.image_views[image_index as usize].handle())
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear_value);

        let color_attachments = [color_attachment];

        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent(),
            })
            .layer_count(1)
            .color_attachments(&color_attachments);

        command_buffer.begin_rendering(&rendering_info);
        command_buffer.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, self.pipeline.unwrap());

        // Set dynamic state
        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent().width as f32)
            .height(swapchain.extent().height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent());

        command_buffer.set_viewport_with_count(&[viewport]);
        command_buffer.set_scissor_with_count(&[scissor]);
        command_buffer.set_primitive_topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        command_buffer.set_cull_mode(vk::CullModeFlags::BACK);
        command_buffer.set_front_face(vk::FrontFace::CLOCKWISE);

        command_buffer.draw(0..3, 0..1);
        command_buffer.end_rendering();

        command_buffer.end();

        // Submit command buffer using the queue RHI
        let submission_info = SubmissionInfo {
            command_buffers: vec![command_buffer.clone()],
            wait_semaphores: vec![image_available.clone()],
            wait_stages: vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            signal_semaphores: vec![render_finished.clone()],
        };

        queue
            .submit(submission_info)
            .expect("Failed to submit draw command buffer");

        // Present
        let _ = swapchain.queue_present(queue.handle(), image_index, &[render_finished.handle()]);
    }

    fn cleanup(&mut self) {
        let device = self.device.as_ref().unwrap();

        unsafe {
            let _ = device.handle().device_wait_idle();

            // Sync objects, command pool, and image views are automatically cleaned up by Drop

            // Destroy pipeline
            if let Some(pipeline) = self.pipeline {
                device.handle().destroy_pipeline(pipeline, None);
            }

            // Destroy pipeline layout
            if let Some(layout) = self.pipeline_layout {
                device.handle().destroy_pipeline_layout(layout, None);
            }

            // Destroy shader modules
            if let Some(module) = self.vertex_shader_module {
                device.handle().destroy_shader_module(module, None);
            }
            if let Some(module) = self.fragment_shader_module {
                device.handle().destroy_shader_module(module, None);
            }
        }
    }
}

impl ApplicationHandler for TriangleDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Vislum Triangle Demo")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            self.window = Some(Arc::new(window));
            self.init_vulkan();
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
                log::info!("Close requested");
                self.cleanup();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(LevelFilter::Info)
        .filter_module("vislum_render", LevelFilter::Debug)
        .parse_default_env()
        .init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = TriangleDemo::new();

    log::info!("Starting triangle demo");
    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}

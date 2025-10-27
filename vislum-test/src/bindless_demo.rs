use std::sync::Arc;
use std::collections::HashMap;

use ash::vk;
use winit::window::Window;

use vislum_rhi::{
    xx::{Buffer, BufferCreateInfo, BufferUsage},
    command::pool::{CommandPool, CommandPoolCreateInfo},
    descriptor::{
        DescriptorPool, DescriptorPoolCreateInfo, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
    },
    descriptor::layout::DescriptorType,
    device::device::Device,
    image::{Image, ImageCreateInfo, ImageDimensions, ImageFormat, ImageView, ImageViewCreateInfo},
    instance::Instance,
    memory::allocator::{MemoryAllocator, MemoryLocation},
    pipeline::{GraphicsPipeline, GraphicsPipelineCreateInfo, ShaderModule},
    queue::Queue,
    surface::Surface,
    swapchain::{Swapchain, SwapchainCreateInfo},
    sync::{Fence, Semaphore},
    AshHandle, VkHandle,
};

/// Bindless demo state - organized and clean
pub struct BindlessDemo {
    // Core Vulkan objects
    device: Arc<Device>,
    instance: Arc<Instance>,
    allocator: Arc<MemoryAllocator>,
    queue: Arc<Queue>,
    
    // Rendering resources
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    command_pool: Arc<CommandPool>,
    command_buffers: Vec<vislum_rhi::command::buffer::CommandBuffer>,
    
    // Bindless resources
    bindless_layout: Arc<DescriptorSetLayout>,
    bindless_pool: Arc<DescriptorPool>,
    bindless_set: DescriptorSet,
    uniform_buffer: Arc<Buffer>,
    texture_array: Arc<Image>,
    texture_array_view: Arc<ImageView>,
    
    // Pipeline
    pipeline: Arc<GraphicsPipeline>,
    
    // Synchronization
    image_available: Vec<Arc<Semaphore>>,
    render_finished: Vec<Arc<Semaphore>>,
    in_flight_fences: Vec<Arc<Fence>>,
    
    // State
    current_frame: usize,
}

impl BindlessDemo {
    pub fn new(
        device: Arc<Device>,
        instance: Arc<Instance>,
        allocator: Arc<MemoryAllocator>,
        queue: Arc<Queue>,
        window: &Window,
    ) -> Self {
        println!("=== Initializing Bindless Demo ===");
        
        // Create surface
        let surface = Surface::new(instance.clone(), window, window);
        println!("✓ Surface created");
        
        // Create swapchain
        let swapchain = Swapchain::new(
            device.clone(),
            &surface,
            SwapchainCreateInfo {
                extent: vk::Extent2D { width: 800, height: 600 },
                format: ImageFormat::B8G8R8A8Srgb,
                present_mode: vk::PresentModeKHR::FIFO,
            },
        );
        println!("✓ Swapchain created with {} images", swapchain.views().len());
        
        // Create command pool
        let command_pool = Arc::new(CommandPool::new(device.clone(), CommandPoolCreateInfo {
            queue_family_index: 0,
            transient: false,
            reset_command_buffer: true,
        }));
        println!("✓ Command pool created");
        
        // Create command buffers
        let command_buffers: Vec<_> = command_pool.allocate_command_buffers(swapchain.views().len() as u32).collect();
        println!("✓ Command buffers created");
        
        // Create bindless descriptor set layout
        let bindless_layout = DescriptorSetLayout::new(
            device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: vec![
                    // Binding 0: Uniform buffer
                    DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: DescriptorType::UniformBuffer,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    },
                    // Binding 1: Bindless texture array
                    DescriptorSetLayoutBinding {
                        binding: 1,
                        descriptor_type: DescriptorType::CombinedImageSampler,
                        descriptor_count: 1000, // Support up to 1000 textures
                        stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    },
                ],
            },
        );
        println!("✓ Bindless descriptor set layout created");
        
        // Create descriptor pool
        let mut pool_sizes = HashMap::new();
        pool_sizes.insert(DescriptorType::UniformBuffer, 10);
        pool_sizes.insert(DescriptorType::CombinedImageSampler, 1000);
        
        let bindless_pool = DescriptorPool::new(
            device.clone(),
            DescriptorPoolCreateInfo {
                max_sets: 10,
                pool_sizes,
            },
        );
        println!("✓ Bindless descriptor pool created");
        
        // Allocate bindless descriptor set
        let bindless_set = bindless_pool
            .allocate(std::iter::once(bindless_layout.clone()))
            .next()
            .unwrap();
        println!("✓ Bindless descriptor set allocated");
        
        // Create uniform buffer
        let uniform_buffer = Arc::new(Buffer::new(
            device.clone(),
            allocator.clone(),
            BufferCreateInfo {
                size: 64, // Simple uniform buffer
                usage: BufferUsage::UNIFORM_BUFFER,
                location: MemoryLocation::CpuToGpu,
            },
        ));
        println!("✓ Uniform buffer created");
        
        // Create texture array (simple 2D array for demo)
        let texture_array = Image::new(
            device.clone(),
            allocator.clone(),
            ImageCreateInfo {
                dimensions: ImageDimensions::D2,
                extent: vk::Extent3D { width: 64, height: 64, depth: 1 },
                format: ImageFormat::R8G8B8A8Srgb,
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            },
        );
        println!("✓ Texture array created");
        
        // Create texture array view
        let texture_array_view = Arc::new(ImageView::new(
            texture_array.clone(),
            ImageViewCreateInfo {
                dimensions: ImageDimensions::D2,
                format: ImageFormat::R8G8B8A8Srgb,
            },
        ));
        println!("✓ Texture array view created");
        
        // Write to bindless descriptor set
        bindless_set.write_buffer(0, &uniform_buffer, 0, uniform_buffer.size());
        bindless_set.write_image(1, &texture_array_view, vk::Sampler::null(), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        println!("✓ Bindless descriptors written");
        
        // Create graphics pipeline (will be implemented with HLSL shaders)
        let pipeline = Self::create_bindless_pipeline(device.clone(), &bindless_layout);
        println!("✓ Bindless graphics pipeline created");
        
        // Create synchronization primitives
        let image_available: Vec<_> = (0..swapchain.views().len())
            .map(|_| Semaphore::new(device.clone()))
            .collect();
        let render_finished: Vec<_> = (0..swapchain.views().len())
            .map(|_| Semaphore::new(device.clone()))
            .collect();
        let in_flight_fences: Vec<_> = (0..swapchain.views().len())
            .map(|_| Fence::new(device.clone(), false))
            .collect();
        println!("✓ Synchronization primitives created");
        
        println!("=== Bindless Demo Initialized ===");
        
        Self {
            device,
            instance,
            allocator,
            queue,
            surface,
            swapchain,
            command_pool,
            command_buffers,
            bindless_layout,
            bindless_pool,
            bindless_set,
            uniform_buffer,
            texture_array,
            texture_array_view,
            pipeline,
            image_available,
            render_finished,
            in_flight_fences,
            current_frame: 0,
        }
    }
    
    fn create_bindless_pipeline(
        device: Arc<Device>,
        layout: &Arc<DescriptorSetLayout>,
    ) -> Arc<GraphicsPipeline> {
        // Load HLSL shaders (will be compiled to SPIR-V)
        let vertex_shader = Self::load_shader(&device, "bindless_vertex");
        let fragment_shader = Self::load_shader(&device, "bindless_fragment");
        
        GraphicsPipeline::new(
            device,
            GraphicsPipelineCreateInfo {
                vertex_shader,
                fragment_shader: fragment_shader,
                descriptor_set_layouts: vec![layout.clone()],
                vertex_buffer: None, // Will be set up for bindless rendering
                color_formats: vec![ImageFormat::B8G8R8A8Srgb],
                depth_format: None,
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            },
        )
    }
    
    fn load_shader(device: &Arc<Device>, name: &str) -> Arc<ShaderModule> {
        // Load compiled SPIR-V shader from disk
        let shader_bytes = Self::get_bindless_shader_spirv(name);
        
        ShaderModule::new(device.clone(), shader_bytes)
    }
    
    fn get_bindless_shader_spirv(name: &str) -> &[u8] {
        match name {
            "bindless_vertex" => include_bytes!("../shaders/bindless_vertex.spv"),
            "bindless_fragment" => include_bytes!("../shaders/bindless_fragment.spv"),
            _ => panic!("Unknown shader: {}", name),
        }
    }
    
    pub fn render(&mut self) {
        let frame = self.current_frame;
        
        // Wait for fence
        self.in_flight_fences[frame].wait(u64::MAX);
        
        // Acquire next image
        let image_index = self.swapchain.acquire_next_image(
            self.image_available[frame].vk_handle()
        ).unwrap_or(0);
        
        // Reset fence
        self.in_flight_fences[frame].reset();
        
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
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
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
        
        // Begin rendering
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.2, 0.3, 0.8, 1.0], // Nice blue color for bindless demo
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
            .layer_count(1);
        
        let color_attachments = [color_attachment];
        let rendering_info = rendering_info.color_attachments(&color_attachments);
        
        cmd.begin_rendering(&rendering_info);
        
        // Bind bindless pipeline
        cmd.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, self.pipeline.as_ref());
        
        // Bind bindless descriptor set
        cmd.bind_descriptor_sets(
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.layout(),
            0,
            &[self.bindless_set.vk_handle()],
        );
        
        // Draw with bindless resources
        cmd.draw(0..3, 0..1); // Simple triangle
        
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
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished[frame].vk_handle()];
        let command_buffers = [cmd.vk_handle()];
        
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        
        unsafe {
            self.device.ash_handle().queue_submit(
                self.queue.vk_handle(),
                &[submit_info],
                self.in_flight_fences[frame].vk_handle(),
            ).expect("Failed to submit queue");
        }
        
        // Present
        self.swapchain.present(
            self.queue.vk_handle(),
            image_index,
            self.render_finished[frame].vk_handle(),
        );
        
        // Next frame
        self.current_frame = (self.current_frame + 1) % self.swapchain.views().len();
    }
    
    pub fn cleanup(&mut self) {
        println!("Cleaning up bindless demo...");
        
        // Wait for device to finish
        unsafe {
            self.device.ash_handle().device_wait_idle().expect("Failed to wait for device");
        }
        
        println!("✓ Bindless demo cleaned up");
    }
}

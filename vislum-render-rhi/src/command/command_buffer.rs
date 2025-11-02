use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, VkHandle, device::Device, sync::{Fence, Semaphore}, queue::Queue};

pub struct CommandPool {
    device: Arc<Device>,
    pool: DebugWrapper<vk::CommandPool>,
}

impl CommandPool {
    /// Creates a new command pool.
    pub fn new(device: Arc<Device>, queue_family_index: u32) -> Arc<Self> {
        let create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let pool = unsafe {
            device.ash_handle().create_command_pool(&create_info, None).unwrap()
        };

        Arc::new(Self {
            device,
            pool: DebugWrapper(pool),
        })
    }

    /// Allocates a command buffer from this pool.
    pub fn allocate(&self, level: vk::CommandBufferLevel) -> CommandBuffer {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool.0)
            .level(level)
            .command_buffer_count(1);

        let command_buffers = unsafe {
            self.device.ash_handle().allocate_command_buffers(&allocate_info).unwrap()
        };

        CommandBuffer {
            device: Arc::clone(&self.device),
            command_buffer: DebugWrapper(command_buffers[0]),
            pool: self.pool.0,
            recording: false,
        }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_command_pool(self.pool.0, None);
        }
    }
}

pub struct CommandBuffer {
    device: Arc<Device>,
    command_buffer: DebugWrapper<vk::CommandBuffer>,
    pool: vk::CommandPool,
    recording: bool,
}

impl CommandBuffer {
    /// Begins recording commands into the command buffer.
    pub fn begin(&mut self, flags: vk::CommandBufferUsageFlags) {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(flags);

        unsafe {
            self.device.ash_handle().begin_command_buffer(self.command_buffer.0, &begin_info).unwrap();
        }

        self.recording = true;
    }

    /// Ends recording commands into the command buffer.
    pub fn end(&mut self) {
        unsafe {
            self.device.ash_handle().end_command_buffer(self.command_buffer.0).unwrap();
        }

        self.recording = false;
    }

    /// Resets the command buffer.
    pub fn reset(&mut self) {
        unsafe {
            self.device.ash_handle().reset_command_buffer(
                self.command_buffer.0,
                vk::CommandBufferResetFlags::empty(),
            ).unwrap();
        }

        self.recording = false;
    }

    /// Begins dynamic rendering.
    pub fn begin_rendering(&self, rendering_info: &vk::RenderingInfo) {
        unsafe {
            use ash::khr::dynamic_rendering::Device;
            let loader = Device::new(self.device.instance().ash_handle(), self.device.ash_handle());
            loader.cmd_begin_rendering(self.command_buffer.0, rendering_info);
        }
    }

    /// Sets the viewport.
    pub fn set_viewport(&self, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe {
            self.device.ash_handle().cmd_set_viewport(
                self.command_buffer.0,
                first_viewport,
                viewports,
            );
        }
    }

    /// Sets the scissor rectangles.
    pub fn set_scissor(&self, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe {
            self.device.ash_handle().cmd_set_scissor(
                self.command_buffer.0,
                first_scissor,
                scissors,
            );
        }
    }

    /// Binds a graphics or compute pipeline.
    pub fn bind_pipeline(&self, pipeline_bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe {
            self.device.ash_handle().cmd_bind_pipeline(
                self.command_buffer.0,
                pipeline_bind_point,
                pipeline,
            );
        }
    }

    /// Binds descriptor sets.
    pub fn bind_descriptor_sets(
        &self,
        pipeline_bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        unsafe {
            self.device.ash_handle().cmd_bind_descriptor_sets(
                self.command_buffer.0,
                pipeline_bind_point,
                layout,
                first_set,
                descriptor_sets,
                dynamic_offsets,
            );
        }
    }

    /// Binds vertex buffers.
    pub fn bind_vertex_buffers(
        &self,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        unsafe {
            self.device.ash_handle().cmd_bind_vertex_buffers(
                self.command_buffer.0,
                first_binding,
                buffers,
                offsets,
            );
        }
    }

    /// Binds vertex buffers from RHI Buffer types.
    pub fn bind_vertex_buffers_buffers(
        &self,
        first_binding: u32,
        buffers: &[&crate::buffer::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        use crate::VkHandle;
        let buffer_handles: Vec<_> = buffers.iter().map(|b| b.vk_handle()).collect();
        self.bind_vertex_buffers(first_binding, &buffer_handles, offsets);
    }

    /// Binds an index buffer.
    pub fn bind_index_buffer(
        &self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        unsafe {
            self.device.ash_handle().cmd_bind_index_buffer(
                self.command_buffer.0,
                buffer,
                offset,
                index_type,
            );
        }
    }

    /// Binds an index buffer from an RHI Buffer type.
    pub fn bind_index_buffer_buffer(
        &self,
        buffer: &crate::buffer::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        use crate::VkHandle;
        self.bind_index_buffer(buffer.vk_handle(), offset, index_type);
    }

    /// Draws indexed primitives.
    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.ash_handle().cmd_draw_indexed(
                self.command_buffer.0,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    /// Ends dynamic rendering.
    pub fn end_rendering(&self) {
        unsafe {
            use ash::khr::dynamic_rendering::Device;
            let loader = Device::new(self.device.instance().ash_handle(), self.device.ash_handle());
            loader.cmd_end_rendering(self.command_buffer.0);
        }
    }

    /// Submits this command buffer to a queue.
    pub fn submit(
        &self,
        queue: &Queue,
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        fence: Option<&Fence>,
    ) {
        let wait_semaphore_handles: Vec<_> = wait_semaphores.iter()
            .map(|s| s.vk_handle())
            .collect();
        let wait_dst_stage_masks: Vec<_> = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; wait_semaphore_handles.len()];

        let signal_semaphore_handles: Vec<_> = signal_semaphores.iter()
            .map(|s| s.vk_handle())
            .collect();

        let command_buffer_handle = self.command_buffer.0;
        let command_buffers = [command_buffer_handle];
        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&command_buffers)
            .wait_semaphores(&wait_semaphore_handles)
            .wait_dst_stage_mask(&wait_dst_stage_masks)
            .signal_semaphores(&signal_semaphore_handles);

        let fence_handle = fence.map(|f| f.vk_handle()).unwrap_or(vk::Fence::null());
        let queue_handle = queue.vk_handle();

        unsafe {
            self.device.ash_handle().queue_submit(
                queue_handle,
                &[submit_info],
                fence_handle,
            ).unwrap();
        }
    }

    /// Returns whether this command buffer is currently recording.
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Returns the device associated with this command buffer.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl VkHandle for CommandBuffer {
    type Handle = vk::CommandBuffer;

    fn vk_handle(&self) -> Self::Handle {
        self.command_buffer.0
    }
}


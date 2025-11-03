use std::sync::Arc;

use ash::vk;
use smallvec::SmallVec;

use crate::command::types::{
    CommandBufferUsageFlags, ImageLayout, IndexType, PipelineBindPoint,
    Rect2D, Viewport,
};
use crate::command::{BufferMemoryBarrier2, ImageMemoryBarrier2, MemoryBarrier2};
use crate::{AshHandle, DebugWrapper, VkHandle, device::Device, vk_enum};

vk_enum! {
    pub enum CommandBufferLevel: ash::vk::CommandBufferLevel {
        PRIMARY => PRIMARY,
        SECONDARY => SECONDARY,
    }
}

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
            device
                .ash_handle()
                .create_command_pool(&create_info, None)
                .unwrap()
        };

        Arc::new(Self {
            device,
            pool: DebugWrapper(pool),
        })
    }

    /// Allocates a command buffer from this pool.
    pub fn allocate(&self, level: CommandBufferLevel) -> RawCommandBuffer {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool.0)
            .level(level.to_vk())
            .command_buffer_count(1);

        let command_buffers = unsafe {
            self.device
                .ash_handle()
                .allocate_command_buffers(&allocate_info)
                .unwrap()
        };

        RawCommandBuffer {
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
            self.device
                .ash_handle()
                .destroy_command_pool(self.pool.0, None);
        }
    }
}

/// A raw command buffer.
///
/// Corresponds to a [`VkCommandBuffer`].
///
/// [`VkCommandBuffer`]: https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkCommandBuffer.html
pub struct RawCommandBuffer {
    device: Arc<Device>,
    command_buffer: DebugWrapper<vk::CommandBuffer>,
    pool: vk::CommandPool,
    recording: bool,
}

impl RawCommandBuffer {
    /// Begins recording commands into the command buffer.
    pub fn begin(&mut self, flags: CommandBufferUsageFlags) {
        let begin_info = vk::CommandBufferBeginInfo::default().flags(flags.to_vk());

        unsafe {
            self.device
                .ash_handle()
                .begin_command_buffer(self.command_buffer.0, &begin_info)
                .unwrap();
        }

        self.recording = true;
    }

    /// Ends recording commands into the command buffer.
    pub fn end(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .end_command_buffer(self.command_buffer.0)
                .unwrap();
        }

        self.recording = false;
    }

    /// Resets the command buffer.
    pub fn reset(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .reset_command_buffer(self.command_buffer.0, vk::CommandBufferResetFlags::empty())
                .unwrap();
        }

        self.recording = false;
    }

    /// Begins dynamic rendering.
    pub fn begin_rendering(&self, rendering_info: &vk::RenderingInfo) {
        unsafe {
            use ash::khr::dynamic_rendering::Device;
            let loader = Device::new(
                self.device.instance().ash_handle(),
                self.device.ash_handle(),
            );
            loader.cmd_begin_rendering(self.command_buffer.0, rendering_info);
        }
    }

    /// Sets the viewport.
    pub fn set_viewport(&self, first_viewport: u32, viewports: impl IntoIterator<Item = Viewport>) {
        let viewports_vk: SmallVec<[vk::Viewport; 8]> =
            viewports.into_iter().map(|v| v.to_vk()).collect();
        unsafe {
            self.device.ash_handle().cmd_set_viewport(
                self.command_buffer.0,
                first_viewport,
                &viewports_vk,
            );
        }
    }

    /// Sets the scissor rectangles.
    pub fn set_scissor(&self, first_scissor: u32, scissors: impl IntoIterator<Item = Rect2D>) {
        let scissors_vk: SmallVec<[vk::Rect2D; 8]> =
            scissors.into_iter().map(|s| s.to_vk()).collect();
        unsafe {
            self.device.ash_handle().cmd_set_scissor(
                self.command_buffer.0,
                first_scissor,
                &scissors_vk,
            );
        }
    }

    /// Binds a graphics or compute pipeline.
    pub fn bind_pipeline(&self, pipeline_bind_point: PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe {
            self.device.ash_handle().cmd_bind_pipeline(
                self.command_buffer.0,
                pipeline_bind_point.to_vk(),
                pipeline,
            );
        }
    }

    /// Binds descriptor sets.
    pub fn bind_descriptor_sets(
        &self,
        pipeline_bind_point: PipelineBindPoint,
        layout: vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: impl IntoIterator<Item = vk::DescriptorSet>,
        dynamic_offsets: impl IntoIterator<Item = u32>,
    ) {
        let descriptor_sets: SmallVec<[vk::DescriptorSet; 4]> =
            descriptor_sets.into_iter().collect();
        let dynamic_offsets: SmallVec<[u32; 4]> = dynamic_offsets.into_iter().collect();
        unsafe {
            self.device.ash_handle().cmd_bind_descriptor_sets(
                self.command_buffer.0,
                pipeline_bind_point.to_vk(),
                layout,
                first_set,
                &descriptor_sets,
                &dynamic_offsets,
            );
        }
    }

    /// Binds vertex buffers.
    pub fn bind_vertex_buffers(
        &self,
        first_binding: u32,
        buffers: &SmallVec<[vk::Buffer; 4]>,
        offsets: &SmallVec<[vk::DeviceSize; 4]>,
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
        buffers: impl IntoIterator<Item = Arc<crate::buffer::Buffer>>,
        offsets: impl IntoIterator<Item = u64>,
    ) {
        use crate::VkHandle;
        let buffer_handles: SmallVec<[vk::Buffer; 4]> =
            buffers.into_iter().map(|b| b.vk_handle()).collect();
        let offsets: SmallVec<[vk::DeviceSize; 4]> = offsets.into_iter().collect();
        self.bind_vertex_buffers(first_binding, &buffer_handles, &offsets);
    }

    /// Binds an index buffer.
    pub fn bind_index_buffer(
        &self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: IndexType,
    ) {
        unsafe {
            self.device.ash_handle().cmd_bind_index_buffer(
                self.command_buffer.0,
                buffer,
                offset,
                index_type.to_vk(),
            );
        }
    }

    /// Binds an index buffer from an RHI Buffer type.
    pub fn bind_index_buffer_buffer(
        &self,
        buffer: &Arc<crate::buffer::Buffer>,
        offset: vk::DeviceSize,
        index_type: IndexType,
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
            let loader = Device::new(
                self.device.instance().ash_handle(),
                self.device.ash_handle(),
            );
            loader.cmd_end_rendering(self.command_buffer.0);
        }
    }

    /// Records a pipeline barrier.
    pub fn pipeline_barrier(
        &self,
        memory_barriers: impl IntoIterator<Item = MemoryBarrier2>,
        buffer_memory_barriers: impl IntoIterator<Item = BufferMemoryBarrier2>,
        image_memory_barriers: impl IntoIterator<Item = ImageMemoryBarrier2>,
    ) {
        let memory_barriers_vk: SmallVec<[vk::MemoryBarrier2; 4]> =
            memory_barriers.into_iter().map(|b| b.to_vk()).collect();
        let buffer_memory_barriers_vk: SmallVec<[vk::BufferMemoryBarrier2; 4]> =
            buffer_memory_barriers
                .into_iter()
                .map(|b| b.to_vk())
                .collect();
        let image_memory_barriers_vk: SmallVec<[vk::ImageMemoryBarrier2; 4]> =
            image_memory_barriers
                .into_iter()
                .map(|b| b.to_vk())
                .collect();

        let barrier_info = vk::DependencyInfo::default()
            .memory_barriers(&memory_barriers_vk)
            .buffer_memory_barriers(&buffer_memory_barriers_vk)
            .image_memory_barriers(&image_memory_barriers_vk);

        unsafe {
            self.device
                .ash_handle()
                .cmd_pipeline_barrier2(self.command_buffer.0, &barrier_info);
        }
    }

    /// Copies data from one buffer to another.
    pub fn copy_buffer(
        &self,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) {
        let copy_region = vk::BufferCopy::default()
            .src_offset(src_offset)
            .dst_offset(dst_offset)
            .size(size);

        unsafe {
            self.device.ash_handle().cmd_copy_buffer(
                self.command_buffer.0,
                src_buffer,
                dst_buffer,
                &[copy_region],
            );
        }
    }

    /// Copies data from a buffer to an image.
    pub fn copy_buffer_to_image(
        &self,
        src_buffer: vk::Buffer,
        dst_image: vk::Image,
        dst_layout: ImageLayout,
        regions: &SmallVec<[vk::BufferImageCopy; 4]>,
    ) {
        unsafe {
            self.device.ash_handle().cmd_copy_buffer_to_image(
                self.command_buffer.0,
                src_buffer,
                dst_image,
                dst_layout.to_vk(),
                regions,
            );
        }
    }

    /// Returns a reference to the device.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl VkHandle for RawCommandBuffer {
    type Handle = vk::CommandBuffer;

    fn vk_handle(&self) -> Self::Handle {
        self.command_buffer.0
    }
}

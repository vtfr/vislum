use std::sync::Arc;
use ash::vk;
use smallvec::SmallVec;

use crate::{
    buffer::Buffer, 
    command::{BufferMemoryBarrier2, ImageMemoryBarrier2, MemoryBarrier2, RawCommandBuffer, types::{BufferImageCopy, CommandBufferUsageFlags, ImageLayout, IndexType, PipelineBindPoint, Rect2D, Viewport}}, 
    image::Image,
};

/// A command encoder that performs automatic resource transitions.
pub struct CommandEncoder {
    command_buffer: RawCommandBuffer,
}

impl CommandEncoder {
    /// Creates a new auto command buffer wrapping a raw command buffer.
    pub fn new(command_buffer: RawCommandBuffer) -> Self {
        Self {
            command_buffer,
        }
    }

    /// Begins recording commands.
    pub fn begin(&mut self, flags: CommandBufferUsageFlags) {
        self.command_buffer.begin(flags);
    }

    /// Ends recording commands.
    pub fn end(&mut self) {
        self.command_buffer.end();
    }

    /// Copies data from one buffer to another.
    pub fn copy_buffer(
        &mut self,
        src_buffer: Arc<Buffer>,
        dst_buffer: Arc<Buffer>,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) {
        use crate::VkHandle;
        self.command_buffer.copy_buffer(
            src_buffer.vk_handle(),
            dst_buffer.vk_handle(),
            src_offset,
            dst_offset,
            size,
        );
    }

    /// Copies data from a buffer to an image.
    /// Caller must ensure the image is in the correct layout before calling this.
    pub fn copy_buffer_to_image(
        &mut self,
        src_buffer: Arc<Buffer>,
        dst_image: Arc<Image>,
        dst_layout: ImageLayout,
        regions: impl IntoIterator<Item = BufferImageCopy>,
    ) {
        use crate::VkHandle;
        let regions_vk: SmallVec<[vk::BufferImageCopy; 4]> = regions.into_iter().map(|r| r.to_vk()).collect();
        self.command_buffer.copy_buffer_to_image(
            src_buffer.vk_handle(),
            dst_image.vk_handle(),
            dst_layout,
            &regions_vk,
        );
    }

    /// Inserts a pipeline barrier.
    pub fn pipeline_barrier(
        &mut self,
        memory_barriers: impl IntoIterator<Item = MemoryBarrier2>,
        buffer_memory_barriers: impl IntoIterator<Item = BufferMemoryBarrier2>,
        image_memory_barriers: impl IntoIterator<Item = ImageMemoryBarrier2>,
    ) {
        self.command_buffer.pipeline_barrier(memory_barriers, buffer_memory_barriers, image_memory_barriers);
    }

    /// Returns a reference to the underlying command buffer.
    pub fn command_buffer(&self) -> &RawCommandBuffer {
        &self.command_buffer
    }

    /// Returns a mutable reference to the underlying command buffer.
    pub fn command_buffer_mut(&mut self) -> &mut RawCommandBuffer {
        &mut self.command_buffer
    }

    /// Consumes the auto command buffer and returns the underlying command buffer.
    pub fn into_command_buffer(self) -> RawCommandBuffer {
        self.command_buffer
    }

    /// Begins dynamic rendering.
    /// Note: RenderingInfo is still a Vulkan type - will be RHI-ified later.
    pub fn begin_rendering(&mut self, rendering_info: &ash::vk::RenderingInfo) {
        self.command_buffer.begin_rendering(rendering_info);
    }

    /// Sets the viewport.
    pub fn set_viewport(&mut self, first_viewport: u32, viewports: impl IntoIterator<Item = Viewport>) {
        self.command_buffer.set_viewport(first_viewport, viewports);
    }

    /// Sets the scissor rectangles.
    pub fn set_scissor(&mut self, first_scissor: u32, scissors: impl IntoIterator<Item = Rect2D>) {
        self.command_buffer.set_scissor(first_scissor, scissors);
    }

    /// Binds a graphics or compute pipeline.
    /// Note: Pipeline is still a Vulkan type - will be RHI-ified later.
    pub fn bind_pipeline(&mut self, pipeline_bind_point: PipelineBindPoint, pipeline: ash::vk::Pipeline) {
        self.command_buffer.bind_pipeline(pipeline_bind_point, pipeline);
    }

    /// Binds descriptor sets.
    /// Note: PipelineLayout and DescriptorSet are still Vulkan types - will be RHI-ified later.
    pub fn bind_descriptor_sets(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        layout: ash::vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: impl IntoIterator<Item = ash::vk::DescriptorSet>,
        dynamic_offsets: impl IntoIterator<Item = u32>,
    ) {
        self.command_buffer.bind_descriptor_sets(
            pipeline_bind_point,
            layout,
            first_set,
            descriptor_sets,
            dynamic_offsets,
        );
    }

    /// Binds vertex buffers.
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: impl IntoIterator<Item = Arc<Buffer>>,
        offsets: impl IntoIterator<Item = u64>,
    ) {
        self.command_buffer.bind_vertex_buffers_buffers(first_binding, buffers, offsets);
    }

    /// Binds an index buffer.
    pub fn bind_index_buffer(
        &mut self,
        buffer: Arc<Buffer>,
        offset: u64,
        index_type: IndexType,
    ) {
        self.command_buffer.bind_index_buffer_buffer(&buffer, offset, index_type);
    }

    /// Draws indexed primitives.
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        self.command_buffer.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        );
    }

    /// Ends dynamic rendering.
    pub fn end_rendering(&mut self) {
        self.command_buffer.end_rendering();
    }
}

impl Into<RawCommandBuffer> for CommandEncoder {
    fn into(self) -> RawCommandBuffer {
        self.command_buffer
    }
}


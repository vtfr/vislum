use ash::vk;
use vislum_render_rhi::{
    command::AutoCommandBuffer,
    command::types::{ImageLayout, AccessFlags2, PipelineStageFlags2},
    image::Image,
    buffer::Buffer,
    VkHandle,
    AshHandle,
};

use crate::graph::tracker::ResourceStateTracker;

/// A command encoder that stores a mutable reference to a resource state tracker
/// and automatically handles resource transitions.
/// 
/// # Deprecated
/// This type is deprecated. Use `AutoCommandBuffer` directly instead, which now handles
/// barriers automatically.
#[deprecated(note = "Use AutoCommandBuffer directly instead")]
pub struct CommandEncoder<'g> {
    auto_command_buffer: AutoCommandBuffer,
    state_tracker: &'g mut ResourceStateTracker,
}

impl<'a> CommandEncoder<'a> {
    /// Creates a new command encoder.
    pub fn new(
        auto_command_buffer: AutoCommandBuffer,
        state_tracker: &'a mut ResourceStateTracker,
    ) -> Self {
        Self {
            auto_command_buffer,
            state_tracker,
        }
    }

    /// Transitions an image to the specified layout and pipeline stages/access.
    pub fn transition_image(
        &mut self,
        image: &Image,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
        src_access: AccessFlags2,
        dst_access: AccessFlags2,
        src_stage: PipelineStageFlags2,
        dst_stage: PipelineStageFlags2,
    ) {
        self.auto_command_buffer.transition_image(
            image,
            old_layout,
            new_layout,
            src_access,
            dst_access,
            src_stage,
            dst_stage,
        );
    }

    /// Returns a mutable reference to the underlying command buffer.
    pub fn command_buffer(&mut self) -> &mut AutoCommandBuffer {
        &mut self.auto_command_buffer
    }
    
    /// Returns an immutable reference to the underlying command buffer.
    pub fn command_buffer_ref(&self) -> &AutoCommandBuffer {
        &self.auto_command_buffer
    }

    /// Returns a mutable reference to the state tracker.
    pub fn state_tracker(&mut self) -> &mut ResourceStateTracker {
        self.state_tracker
    }
    
    pub fn auto_command_buffer(self) -> AutoCommandBuffer {
        self.auto_command_buffer
    }

    /// Copies data from one buffer to another.
    pub fn copy_buffer(
        &mut self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) {
        self.auto_command_buffer.copy_buffer(
            src_buffer,
            dst_buffer,
            src_offset,
            dst_offset,
            size,
        );
    }

    /// Copies data from a buffer to an image.
    pub fn copy_buffer_to_image(
        &mut self,
        src_buffer: &Buffer,
        dst_image: &Image,
        dst_layout: ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        self.auto_command_buffer.copy_buffer_to_image(
            src_buffer,
            dst_image,
            dst_layout,
            regions,
        );
    }
}


use std::sync::Arc;
use std::collections::HashMap;

use ash::vk;

use crate::{
    AshHandle, VkHandle,
    device::Device,
    image::Image,
    buffer::Buffer,
    command::CommandBuffer,
    command::types::{CommandBufferUsageFlags, ImageLayout, AccessFlags2, PipelineStageFlags2, PipelineBindPoint, IndexType, Viewport, Rect2D},
};

/// Tracks the layout and access state of images for automatic barrier insertion.
struct ImageState {
    layout: ImageLayout,
    access_mask: AccessFlags2,
}

/// A command buffer that automatically inserts memory barriers based on resource usage.
pub struct AutoCommandBuffer {
    command_buffer: CommandBuffer,
    device: Arc<Device>,
    image_states: HashMap<ash::vk::Image, ImageState>,
}

impl AutoCommandBuffer {
    /// Creates a new auto command buffer wrapping a raw command buffer.
    pub fn new(command_buffer: CommandBuffer) -> Self {
        let device = command_buffer.device().clone();

        Self {
            command_buffer,
            device,
            image_states: HashMap::new(),
        }
    }

    /// Begins recording commands.
    pub fn begin(&mut self, flags: CommandBufferUsageFlags) {
        self.command_buffer.begin(flags);
        self.image_states.clear();
    }

    /// Ends recording commands.
    pub fn end(&mut self) {
        self.command_buffer.end();
    }

    /// Records a pipeline barrier.
    #[allow(unused_variables)]
    pub fn pipeline_barrier(
        &mut self,
        src_stage: vk::PipelineStageFlags2,
        dst_stage: vk::PipelineStageFlags2,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier2],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier2],
        image_memory_barriers: &[vk::ImageMemoryBarrier2],
    ) {
        let barrier_info = vk::DependencyInfo::default()
            .memory_barriers(memory_barriers)
            .buffer_memory_barriers(buffer_memory_barriers)
            .image_memory_barriers(image_memory_barriers)
            .dependency_flags(dependency_flags);

        unsafe {
            self.device.ash_handle().cmd_pipeline_barrier2(
                self.command_buffer.vk_handle(),
                &barrier_info,
            );
        }
    }

    /// Transitions an image to a new layout with automatic barrier insertion.
    #[allow(unused_variables)]
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
        let image_handle = image.vk_handle();
        let current_state = self.image_states.get(&image_handle);

        // If we have a tracked state and it matches, skip the barrier
        if let Some(state) = current_state {
            if state.layout == old_layout && state.access_mask == src_access {
                // Update tracked state
                self.image_states.insert(image_handle, ImageState {
                    layout: new_layout,
                    access_mask: dst_access,
                });
                return;
            }
        }

        // Insert barrier
        let barrier = vk::ImageMemoryBarrier2::default()
            .old_layout(old_layout.to_vk())
            .new_layout(new_layout.to_vk())
            .src_access_mask(src_access.to_vk())
            .dst_access_mask(dst_access.to_vk())
            .src_stage_mask(src_stage.to_vk())
            .dst_stage_mask(dst_stage.to_vk())
            .image(image_handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1));

        self.pipeline_barrier(
            src_stage.to_vk(),
            dst_stage.to_vk(),
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );

        // Update tracked state
        self.image_states.insert(image_handle, ImageState {
            layout: new_layout,
            access_mask: dst_access,
        });
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
        let copy_region = vk::BufferCopy::default()
            .src_offset(src_offset)
            .dst_offset(dst_offset)
            .size(size);

        unsafe {
            self.device.ash_handle().cmd_copy_buffer(
                self.command_buffer.vk_handle(),
                src_buffer.vk_handle(),
                dst_buffer.vk_handle(),
                &[copy_region],
            );
        }
    }

    /// Copies data from a buffer to an image.
    /// Automatically inserts barriers to transition the image to the destination layout.
    pub fn copy_buffer_to_image(
        &mut self,
        src_buffer: &Buffer,
        dst_image: &Image,
        dst_layout: ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        let image_handle = dst_image.vk_handle();
        
        // Check if we need to transition the image layout
        if let Some(current_state) = self.image_states.get(&image_handle) {
            if current_state.layout != dst_layout {
                // Transition to transfer dst layout first
                self.transition_image(
                    dst_image,
                    current_state.layout,
                    dst_layout,
                    current_state.access_mask,
                    AccessFlags2::TRANSFER_WRITE,
                    PipelineStageFlags2::ALL_COMMANDS,
                    PipelineStageFlags2::TRANSFER,
                );
            }
        } else {
            // Image not tracked yet, transition from UNDEFINED
            self.transition_image(
                dst_image,
                ImageLayout::Undefined,
                dst_layout,
                AccessFlags2::NONE,
                AccessFlags2::TRANSFER_WRITE,
                PipelineStageFlags2::TOP_OF_PIPE,
                PipelineStageFlags2::TRANSFER,
            );
        }
        
        unsafe {
            self.device.ash_handle().cmd_copy_buffer_to_image(
                self.command_buffer.vk_handle(),
                src_buffer.vk_handle(),
                dst_image.vk_handle(),
                dst_layout.to_vk(),
                regions,
            );
        }
        
        // Update tracked state
        self.image_states.insert(image_handle, ImageState {
            layout: dst_layout,
            access_mask: AccessFlags2::TRANSFER_WRITE,
        });
    }

    /// Returns a reference to the underlying command buffer.
    pub fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }

    /// Returns a mutable reference to the underlying command buffer.
    pub fn command_buffer_mut(&mut self) -> &mut CommandBuffer {
        &mut self.command_buffer
    }

    /// Consumes the auto command buffer and returns the underlying command buffer.
    pub fn into_command_buffer(self) -> CommandBuffer {
        self.command_buffer
    }

    /// Begins dynamic rendering.
    pub fn begin_rendering(&mut self, rendering_info: &vk::RenderingInfo) {
        self.command_buffer.begin_rendering(rendering_info);
    }

    /// Sets the viewport.
    pub fn set_viewport(&mut self, first_viewport: u32, viewports: &[Viewport]) {
        self.command_buffer.set_viewport(first_viewport, viewports);
    }

    /// Sets the scissor rectangles.
    pub fn set_scissor(&mut self, first_scissor: u32, scissors: &[Rect2D]) {
        self.command_buffer.set_scissor(first_scissor, scissors);
    }

    /// Binds a graphics or compute pipeline.
    pub fn bind_pipeline(&mut self, pipeline_bind_point: PipelineBindPoint, pipeline: vk::Pipeline) {
        self.command_buffer.bind_pipeline(pipeline_bind_point, pipeline);
    }

    /// Binds descriptor sets.
    pub fn bind_descriptor_sets(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        layout: vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
        dynamic_offsets: &[u32],
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
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        self.command_buffer.bind_vertex_buffers(first_binding, buffers, offsets);
    }

    /// Binds vertex buffers from RHI Buffer types.
    pub fn bind_vertex_buffers_buffers(
        &mut self,
        first_binding: u32,
        buffers: &[&Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        self.command_buffer.bind_vertex_buffers_buffers(first_binding, buffers, offsets);
    }

    /// Binds an index buffer.
    pub fn bind_index_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: IndexType,
    ) {
        self.command_buffer.bind_index_buffer(buffer, offset, index_type);
    }

    /// Binds an index buffer from an RHI Buffer type.
    pub fn bind_index_buffer_buffer(
        &mut self,
        buffer: &Buffer,
        offset: vk::DeviceSize,
        index_type: IndexType,
    ) {
        self.command_buffer.bind_index_buffer_buffer(buffer, offset, index_type);
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


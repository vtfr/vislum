use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle};

use super::pool::CommandPool;

pub struct CommandBuffer {
    pool: Arc<CommandPool>,
    buffer: vk::CommandBuffer,
}

impl VkHandle for CommandBuffer {
    type Handle = vk::CommandBuffer;

    fn vk_handle(&self) -> Self::Handle {
        self.buffer
    }
}

impl CommandBuffer {
    pub(crate) fn new(pool: Arc<CommandPool>, buffer: vk::CommandBuffer) -> Self {
        Self { pool, buffer }
    }

    pub fn begin(&self, one_time_submit: bool) {
        let mut flags = vk::CommandBufferUsageFlags::empty();
        if one_time_submit {
            flags |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;
        }

        let begin_info = vk::CommandBufferBeginInfo::default().flags(flags);

        unsafe {
            self.pool
                .device()
                .ash_handle()
                .begin_command_buffer(self.buffer, &begin_info)
                .expect("Failed to begin command buffer");
        }
    }

    pub fn end(&self) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .end_command_buffer(self.buffer)
                .expect("Failed to end command buffer");
        }
    }

    pub fn pipeline_barrier(
        &self,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            self.pool.device().ash_handle().cmd_pipeline_barrier(
                self.buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            );
        }
    }

    pub fn reset(&self, release_resources: bool) {
        let flags = if release_resources {
            vk::CommandBufferResetFlags::RELEASE_RESOURCES
        } else {
            vk::CommandBufferResetFlags::empty()
        };

        unsafe {
            self.pool
                .device()
                .ash_handle()
                .reset_command_buffer(self.buffer, flags)
                .expect("Failed to reset command buffer");
        }
    }

    // Dynamic Rendering
    pub fn begin_rendering(&self, rendering_info: &vk::RenderingInfo) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_begin_rendering(self.buffer, rendering_info);
        }
    }

    pub fn end_rendering(&self) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_end_rendering(self.buffer);
        }
    }

    // Dynamic State
    pub fn set_viewport(&self, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe {
            self.pool.device().ash_handle().cmd_set_viewport(
                self.buffer,
                first_viewport,
                viewports,
            );
        }
    }

    pub fn set_scissor(&self, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_scissor(self.buffer, first_scissor, scissors);
        }
    }

    pub fn set_line_width(&self, line_width: f32) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_line_width(self.buffer, line_width);
        }
    }

    pub fn set_depth_bias(
        &self,
        depth_bias_constant_factor: f32,
        depth_bias_clamp: f32,
        depth_bias_slope_factor: f32,
    ) {
        unsafe {
            self.pool.device().ash_handle().cmd_set_depth_bias(
                self.buffer,
                depth_bias_constant_factor,
                depth_bias_clamp,
                depth_bias_slope_factor,
            );
        }
    }

    pub fn set_blend_constants(&self, blend_constants: &[f32; 4]) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_blend_constants(self.buffer, blend_constants);
        }
    }

    pub fn set_depth_bounds(&self, min_depth_bounds: f32, max_depth_bounds: f32) {
        unsafe {
            self.pool.device().ash_handle().cmd_set_depth_bounds(
                self.buffer,
                min_depth_bounds,
                max_depth_bounds,
            );
        }
    }

    pub fn set_stencil_compare_mask(&self, face_mask: vk::StencilFaceFlags, compare_mask: u32) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_stencil_compare_mask(self.buffer, face_mask, compare_mask);
        }
    }

    pub fn set_stencil_write_mask(&self, face_mask: vk::StencilFaceFlags, write_mask: u32) {
        unsafe {
            self.pool.device().ash_handle().cmd_set_stencil_write_mask(
                self.buffer,
                face_mask,
                write_mask,
            );
        }
    }

    pub fn set_stencil_reference(&self, face_mask: vk::StencilFaceFlags, reference: u32) {
        unsafe {
            self.pool.device().ash_handle().cmd_set_stencil_reference(
                self.buffer,
                face_mask,
                reference,
            );
        }
    }

    // Extended Dynamic State (VK_EXT_extended_dynamic_state)
    pub fn set_cull_mode(&self, cull_mode: vk::CullModeFlags) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_cull_mode(self.buffer, cull_mode);
        }
    }

    pub fn set_front_face(&self, front_face: vk::FrontFace) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_front_face(self.buffer, front_face);
        }
    }

    pub fn set_primitive_topology(&self, primitive_topology: vk::PrimitiveTopology) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_primitive_topology(self.buffer, primitive_topology);
        }
    }

    pub fn set_viewport_with_count(&self, viewports: &[vk::Viewport]) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_viewport_with_count(self.buffer, viewports);
        }
    }

    pub fn set_scissor_with_count(&self, scissors: &[vk::Rect2D]) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_scissor_with_count(self.buffer, scissors);
        }
    }

    pub fn set_depth_test_enable(&self, depth_test_enable: bool) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_depth_test_enable(self.buffer, depth_test_enable);
        }
    }

    pub fn set_depth_write_enable(&self, depth_write_enable: bool) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_depth_write_enable(self.buffer, depth_write_enable);
        }
    }

    pub fn set_depth_compare_op(&self, depth_compare_op: vk::CompareOp) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_depth_compare_op(self.buffer, depth_compare_op);
        }
    }

    pub fn set_depth_bounds_test_enable(&self, depth_bounds_test_enable: bool) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_depth_bounds_test_enable(self.buffer, depth_bounds_test_enable);
        }
    }

    pub fn set_stencil_test_enable(&self, stencil_test_enable: bool) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .cmd_set_stencil_test_enable(self.buffer, stencil_test_enable);
        }
    }

    pub fn set_stencil_op(
        &self,
        face_mask: vk::StencilFaceFlags,
        fail_op: vk::StencilOp,
        pass_op: vk::StencilOp,
        depth_fail_op: vk::StencilOp,
        compare_op: vk::CompareOp,
    ) {
        unsafe {
            self.pool.device().ash_handle().cmd_set_stencil_op(
                self.buffer,
                face_mask,
                fail_op,
                pass_op,
                depth_fail_op,
                compare_op,
            );
        }
    }

    // Drawing
    pub fn bind_pipeline(
        &self,
        bind_point: vk::PipelineBindPoint,
        pipeline: &impl VkHandle<Handle = vk::Pipeline>,
    ) {
        unsafe {
            self.pool.device().ash_handle().cmd_bind_pipeline(
                self.buffer,
                bind_point,
                pipeline.vk_handle(),
            );
        }
    }

    pub fn bind_vertex_buffer(&self, buffer: &impl VkHandle<Handle = vk::Buffer>, offset: u64) {
        let buffers = [buffer.vk_handle()];
        let offsets = [offset];
        unsafe {
            self.pool.device().ash_handle().cmd_bind_vertex_buffers(
                self.buffer,
                0,
                &buffers,
                &offsets,
            );
        }
    }

    pub fn bind_index_buffer(
        &self,
        buffer: &impl VkHandle<Handle = vk::Buffer>,
        offset: u64,
        index_type: vk::IndexType,
    ) {
        unsafe {
            self.pool.device().ash_handle().cmd_bind_index_buffer(
                self.buffer,
                buffer.vk_handle(),
                offset,
                index_type,
            );
        }
    }

    pub fn bind_descriptor_sets(
        &self,
        pipeline_bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
    ) {
        unsafe {
            self.pool.device().ash_handle().cmd_bind_descriptor_sets(
                self.buffer,
                pipeline_bind_point,
                layout,
                first_set,
                descriptor_sets,
                &[],
            );
        }
    }

    pub fn draw(&self, vertices: std::ops::Range<u32>, instances: std::ops::Range<u32>) {
        unsafe {
            self.pool.device().ash_handle().cmd_draw(
                self.buffer,
                vertices.end - vertices.start,
                instances.end - instances.start,
                vertices.start,
                instances.start,
            );
        }
    }

    pub fn draw_indexed(
        &self,
        indices: std::ops::Range<u32>,
        vertex_offset: i32,
        instances: std::ops::Range<u32>,
    ) {
        unsafe {
            self.pool.device().ash_handle().cmd_draw_indexed(
                self.buffer,
                indices.end - indices.start,
                instances.end - instances.start,
                indices.start,
                vertex_offset,
                instances.start,
            );
        }
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.pool
                .device()
                .ash_handle()
                .free_command_buffers(self.pool.vk_handle(), &[self.buffer]);
        }
    }
}

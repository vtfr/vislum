use std::sync::Arc;

use smallvec::SmallVec;
use vulkano::{
    buffer::Subbuffer,
    command_buffer::{RecordingCommandBuffer, RenderingAttachmentInfo, RenderingInfo},
    image::Image,
    pipeline::GraphicsPipeline,
    sync::{AccessFlags, ImageMemoryBarrier, PipelineStages},
};

use crate::graph::tracker::ResourceStateTracker;
use crate::resource::mesh::Mesh;

/// A command encoder that stores a mutable reference to a resource state tracker
/// and automatically handles resource transitions.
pub struct CommandEncoder<'g> {
    command_buffer: &'g mut RecordingCommandBuffer,
    state_tracker: &'g mut ResourceStateTracker,
    pending_barriers: SmallVec<[ImageMemoryBarrier; 16]>,
}

impl<'a> CommandEncoder<'a> {
    /// Creates a new command encoder.
    pub fn new(
        command_buffer: &'a mut RecordingCommandBuffer,
        state_tracker: &'a mut ResourceStateTracker,
    ) -> Self {
        Self {
            command_buffer,
            state_tracker,
            pending_barriers: SmallVec::new(),
        }
    }

    /// Transitions an image to the specified layout and pipeline stages/access.
    /// The transition will be deferred until `flush_barriers()` is called.
    pub fn transition_image(
        &mut self,
        image: Arc<Image>,
        dst_stages: PipelineStages,
        dst_access: AccessFlags,
        new_layout: vulkano::image::ImageLayout,
    ) {
        if let Some(barrier) = self
            .state_tracker
            .transition_image_layout(image, dst_stages, dst_access, new_layout)
        {
            self.pending_barriers.push(barrier);
        }
    }

    /// Flushes all pending barriers to the command buffer.
    pub fn flush_barriers(&mut self) {
        if !self.pending_barriers.is_empty() {
            let barrier_info = vulkano::sync::DependencyInfo {
                image_memory_barriers: self.pending_barriers.iter().map(|b| b.clone()).collect(),
                ..Default::default()
            };
            unsafe {
                self.command_buffer.pipeline_barrier(&barrier_info).unwrap();
            }
            self.pending_barriers.clear();
        }
    }

    /// Returns a mutable reference to the underlying command buffer.
    pub fn command_buffer(&mut self) -> &mut RecordingCommandBuffer {
        self.command_buffer
    }

    /// Returns an immutable reference to the underlying command buffer.
    pub fn command_buffer_ref(&self) -> &RecordingCommandBuffer {
        self.command_buffer
    }

    /// Returns a mutable reference to the state tracker.
    pub fn state_tracker(&mut self) -> &mut ResourceStateTracker {
        self.state_tracker
    }

    /// Begins dynamic rendering with color and depth attachments.
    pub fn begin_rendering(&mut self, rendering_info: RenderingInfo) {
        self.flush_barriers();
        unsafe {
            self.command_buffer.begin_rendering(&rendering_info).unwrap();
        }
    }

    /// Ends the current dynamic rendering.
    pub fn end_rendering(&mut self) {
        unsafe {
            self.command_buffer.end_rendering().unwrap();
        }
    }

    /// Binds a graphics pipeline.
    pub fn bind_pipeline(&mut self, pipeline: Arc<GraphicsPipeline>) {
        self.flush_barriers();
        unsafe {
            self.command_buffer.bind_pipeline_graphics(&pipeline).unwrap();
        }
    }

    /// Binds vertex buffers.
    pub fn bind_vertex_buffers(&mut self, first_binding: u32, buffers: &[Subbuffer<[u8]>]) {
        unsafe {
            self.command_buffer
                .bind_vertex_buffers(first_binding, buffers)
                .unwrap();
        }
    }

    /// Binds a mesh's buffers for drawing.
    pub fn bind_mesh(&mut self, mesh: &Mesh) {
        let vertex_buffer_bytes = mesh.vertex_buffer.as_bytes();
        self.bind_vertex_buffers(0, &[vertex_buffer_bytes.clone()]);
        unsafe {
            self.command_buffer.bind_index_buffer(&mesh.index_buffer).unwrap();
        }
    }

    /// Draws primitives using vertex data from the bound vertex buffers.
    pub fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            self.command_buffer
                .draw(vertex_count, instance_count, first_vertex, first_instance)
                .unwrap();
        }
    }

    /// Draws primitives using indexed data from the bound index buffer.
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.command_buffer
                .draw_indexed(index_count, instance_count, first_index, vertex_offset, first_instance)
                .unwrap();
        }
    }

    /// Draws primitives using a mesh.
    pub fn draw_mesh(&mut self, mesh: &Mesh, instance_count: u32, first_instance: u32) {
        self.bind_mesh(mesh);
        unsafe {
            self.command_buffer
                .draw_indexed(
                    mesh.index_buffer.len() as u32,
                    instance_count,
                    0,
                    0,
                    first_instance,
                )
                .unwrap();
        }
    }
}

impl<'a> Drop for CommandEncoder<'a> {
    fn drop(&mut self) {
        // Automatically flush any pending barriers when the encoder is dropped
        self.flush_barriers();
    }
}


use std::{borrow::Cow, sync::Arc};

use crate::graph::{ExecuteContext, FrameNode, PrepareContext};
use vislum_render_rhi::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command::{AccessFlags2, BufferMemoryBarrier2, PipelineStageFlags2},
    memory::{MemoryAllocator, MemoryLocation},
};

/// A vertex with position, normal, and UV coordinates.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// A mesh containing vertex and index data.
pub struct Mesh {
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    vertex_count: usize,
    index_count: usize,
}

impl Mesh {
    pub fn new(
        device: Arc<vislum_render_rhi::device::Device>,
        allocator: Arc<MemoryAllocator>,
        vertices: impl IntoIterator<Item = Vertex>,
        indices: impl IntoIterator<Item = u16>,
    ) -> (Self, MeshUploadTask) {
        let vertices = vertices.into_iter().collect::<Vec<_>>();
        let indices = indices.into_iter().collect::<Vec<_>>();

        let vertex_count = vertices.len();
        let index_count = indices.len();

        let vertex_data_size = (vertex_count * std::mem::size_of::<Vertex>()) as u64;
        let index_data_size = (index_count * std::mem::size_of::<u16>()) as u64;

        // Create GPU buffers
        let vertex_buffer = Buffer::new(
            device.clone(),
            allocator.clone(),
            BufferCreateInfo {
                size: vertex_data_size,
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
            },
            MemoryLocation::GpuOnly,
        );

        let index_buffer = Buffer::new(
            device.clone(),
            allocator.clone(),
            BufferCreateInfo {
                size: index_data_size,
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
            },
            MemoryLocation::GpuOnly,
        );

        // Create staging buffers
        let vertex_staging = Buffer::new_staging_with_data(
            device.clone(),
            allocator.clone(),
            bytemuck::cast_slice(&vertices),
        );

        let index_staging =
            Buffer::new_staging_with_data(device, allocator, bytemuck::cast_slice(&indices));

        let mesh = Mesh {
            vertex_buffer: vertex_buffer.clone(),
            index_buffer: index_buffer.clone(),
            vertex_count,
            index_count,
        };

        let upload_task = MeshUploadTask {
            vertex_buffer,
            index_buffer,
            vertex_staging,
            index_staging,
            vertex_count,
            index_count,
        };

        (mesh, upload_task)
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    #[inline]
    pub fn index_count(&self) -> usize {
        self.index_count
    }

    #[inline]
    pub fn vertex_buffer(&self) -> Arc<Buffer> {
        self.vertex_buffer.clone()
    }

    #[inline]
    pub fn index_buffer(&self) -> Arc<Buffer> {
        self.index_buffer.clone()
    }
}

pub struct MeshUploadTask {
    vertex_buffer: Arc<Buffer>,
    index_buffer: Arc<Buffer>,
    vertex_staging: Arc<Buffer>,
    index_staging: Arc<Buffer>,
    vertex_count: usize,
    index_count: usize,
}

impl FrameNode for MeshUploadTask {
    fn name(&self) -> Cow<'static, str> {
        "upload_mesh".into()
    }

    fn prepare(
        &self,
        _context: &mut PrepareContext,
    ) -> Box<dyn FnMut(&mut ExecuteContext) + 'static> {
        let vertex_buffer = self.vertex_buffer.clone();
        let index_buffer = self.index_buffer.clone();
        let vertex_staging = self.vertex_staging.clone();
        let index_staging = self.index_staging.clone();
        let vertex_size = (self.vertex_count * std::mem::size_of::<Vertex>()) as u64;
        let index_size = (self.index_count * std::mem::size_of::<u16>()) as u64;

        Box::new(move |execute_context| {
            let cmd = &mut execute_context.command_buffer;

            // Transition the vertex bufer and index buffer to be read.
            cmd.pipeline_barrier(
                std::iter::empty(),
                [vertex_buffer.clone(), index_buffer.clone()]
                    .into_iter()
                    .map(|buffer| BufferMemoryBarrier2 {
                        size: buffer.size(),
                        buffer,
                        src_stage_mask: PipelineStageFlags2::TOP_OF_PIPE,
                        src_access_mask: AccessFlags2::NONE,
                        dst_stage_mask: PipelineStageFlags2::TRANSFER,
                        dst_access_mask: AccessFlags2::TRANSFER_WRITE,
                        offset: 0,
                    }),
                std::iter::empty(),
            );

            // Transition the vertex staging buffer and index staging buffer to
            // be read from.
            cmd.pipeline_barrier(
                std::iter::empty(),
                [vertex_staging.clone(), index_staging.clone()]
                    .into_iter()
                    .map(|buffer| BufferMemoryBarrier2 {
                        size: buffer.size(),
                        buffer,
                        src_stage_mask: PipelineStageFlags2::TOP_OF_PIPE,
                        src_access_mask: AccessFlags2::NONE,
                        dst_stage_mask: PipelineStageFlags2::TRANSFER,
                        dst_access_mask: AccessFlags2::TRANSFER_READ,
                        offset: 0,
                    }),
                std::iter::empty(),
            );

            // Copy vertex buffer
            cmd.copy_buffer(
                vertex_staging.clone(),
                vertex_buffer.clone(),
                0,
                0,
                vertex_size,
            );

            // Copy index buffer
            cmd.copy_buffer(
                index_staging.clone(),
                index_buffer.clone(),
                0,
                0,
                index_size,
            );

            // Transition the vertex buffer and index buffer to be used as vertex and index buffers.
            cmd.pipeline_barrier(
                std::iter::empty(),
                [
                    (
                        vertex_buffer.clone(),
                        PipelineStageFlags2::VERTEX_INPUT,
                        AccessFlags2::VERTEX_ATTRIBUTE_READ,
                    ),
                    (
                        index_buffer.clone(),
                        PipelineStageFlags2::VERTEX_INPUT,
                        AccessFlags2::INDEX_READ,
                    ),
                ]
                .into_iter()
                .map(|(buffer, dst_stage_mask, dst_access_mask)| BufferMemoryBarrier2 {
                    size: buffer.size(),
                    buffer,
                    src_stage_mask: PipelineStageFlags2::TRANSFER,
                    src_access_mask: AccessFlags2::TRANSFER_WRITE,
                    dst_stage_mask,
                    dst_access_mask,
                    offset: 0,
                }),
                std::iter::empty(),
            );
        })
    }
}

use std::{borrow::Cow, sync::Arc};

use vislum_render_rhi::{buffer::{Buffer, BufferCreateInfo, BufferUsage}, memory::{MemoryAllocator, MemoryLocation}};
use crate::graph::{FrameNode, PrepareContext, ExecuteContext};

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
    ) -> (Arc<Self>, MeshUploadTask) {
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

        let index_staging = Buffer::new_staging_with_data(
            device,
            allocator,
            bytemuck::cast_slice(&indices),
        );

        let mesh = Arc::new(Mesh {
            vertex_buffer,
            index_buffer,
            vertex_count,
            index_count,
        });

        let upload_task = MeshUploadTask {
            mesh: mesh.clone(),
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
    mesh: Arc<Mesh>,
    vertex_staging: Arc<Buffer>,
    index_staging: Arc<Buffer>,
    vertex_count: usize,
    index_count: usize,
}

impl FrameNode for MeshUploadTask {
    fn name(&self) -> Cow<'static, str> {
        "upload_mesh".into()
    }

    fn prepare(&self, _context: &mut PrepareContext) -> Box<dyn FnMut(&mut ExecuteContext<'_>) + 'static> {
        let vertex_buffer = self.mesh.vertex_buffer.clone();
        let index_buffer = self.mesh.index_buffer.clone();
        let vertex_staging = self.vertex_staging.clone();
        let index_staging = self.index_staging.clone();
        let vertex_size = (self.vertex_count * std::mem::size_of::<Vertex>()) as u64;
        let index_size = (self.index_count * std::mem::size_of::<u16>()) as u64;

        Box::new(move |execute_context| {
            // Copy vertex buffer
            execute_context.command_encoder.copy_buffer(
                &vertex_staging,
                &vertex_buffer,
                0,
                0,
                vertex_size,
            );

            // Copy index buffer
            execute_context.command_encoder.copy_buffer(
                &index_staging,
                &index_buffer,
                0,
                0,
                index_size,
            );
        })
    }
}
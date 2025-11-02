use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
};

/// A vertex with position, normal, and UV coordinates.
#[repr(C)]
#[derive(BufferContents, Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// A mesh containing vertex and index data.
pub struct Mesh {
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub index_buffer: Subbuffer<[u32]>,
}

impl Mesh {
    /// Creates a new mesh with empty device-local buffers.
    /// Data must be uploaded separately using the upload system.
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        vertex_count: usize,
        index_count: usize,
    ) -> Self {
        // Create vertex buffer
        let vertex_buffer = Buffer::new_slice::<Vertex>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            vertex_count as u64,
        ).expect("Failed to create vertex buffer");

        // Create index buffer
        let index_buffer = Buffer::new_slice::<u32>(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            index_count as u64,
        ).expect("Failed to create index buffer");

        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}
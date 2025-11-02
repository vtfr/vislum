use std::sync::Arc;

use ash::vk;
use vislum_render_rhi::{buffer::Buffer, memory::MemoryAllocator};

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
    pub vertex_buffer: Arc<Buffer>,
    pub index_buffer: Arc<Buffer>,
    pub vertex_count: usize,
    pub index_count: usize,
}

impl Mesh {
    /// Creates a new mesh with empty device-local buffers.
    /// Data must be uploaded separately using the upload system.
    pub fn new(
        device: Arc<vislum_render_rhi::device::Device>,
        allocator: Arc<MemoryAllocator>,
        vertex_count: usize,
        index_count: usize,
    ) -> Self {
        use vislum_render_rhi::buffer::BufferCreateInfo;

        // Create vertex buffer
        let vertex_buffer = Buffer::new(
            device.clone(),
            allocator.clone(),
            BufferCreateInfo {
                size: (vertex_count * std::mem::size_of::<Vertex>()) as u64,
                usage: vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                flags: vk::BufferCreateFlags::empty(),
            },
        );

        // Create index buffer
        let index_buffer = Buffer::new(
            device,
            allocator,
            BufferCreateInfo {
                size: (index_count * std::mem::size_of::<u32>()) as u64,
                usage: vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                flags: vk::BufferCreateFlags::empty(),
            },
        );

        Self {
            vertex_buffer,
            index_buffer,
            vertex_count,
            index_count,
        }
    }
}
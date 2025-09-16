use bytemuck::{Pod, Zeroable};

use crate::{device::RenderDevice, resource::{ResourceId, ResourceStorage}};

/// A vertex of a mesh.
#[derive(Clone, Copy, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 3],
}

pub type Index = u32;

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Mesh {
    /// Gets the vertex buffer of the mesh.
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// Gets the index buffer of the mesh.
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    /// Gets the number of indices in the mesh.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

pub struct MeshDescriptor<'a> {
    vertices: &'a [Vertex],
    indices: &'a [Index],
}

pub struct MeshManager {
    device: RenderDevice,
    meshes: ResourceStorage<Mesh>,
}

impl MeshManager {
    pub fn new(device: RenderDevice) -> Self {
        Self { device, meshes: Default::default() }
    }

    pub fn create(&mut self, descriptor: MeshDescriptor) -> ResourceId<Mesh> {
        use wgpu::util::DeviceExt;

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(descriptor.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(descriptor.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh = Mesh {
            vertices: descriptor.vertices.to_vec(),
            indices: descriptor.indices.to_vec(),
            vertex_buffer,
            index_buffer,
        };

        self.meshes.insert(mesh)
    }
}

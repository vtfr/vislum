use vislum_system::Resource;
use wgpu::util::DeviceExt;

use crate::cache::types::RenderDevice;
use crate::cache::storage::{Handle, IntoResourceId, ResourceStorage};

pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    buffer: wgpu::Buffer,
}

impl Mesh {
    /// Gets the vertices of the mesh.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Gets the indices of the mesh.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Gets the vertex buffer of the mesh.
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

pub struct Vertex {
    pub position: [f32; 3],
}

#[derive(Resource)]
pub struct MeshManager {
    device: RenderDevice,
    meshes: ResourceStorage<Mesh>,
}

impl MeshManager {
    pub fn new(device: RenderDevice) -> Self {
        Self { device, meshes: ResourceStorage::new() }
    }

    pub fn create(&mut self, descriptor: MeshDescriptor) -> Handle<Mesh> {
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.meshes.insert(Mesh {
            vertices: descriptor.vertices,
            indices: descriptor.indices,
            buffer,
        })
    }
    
    pub fn get(&self, id: impl IntoResourceId<Mesh>) -> Option<&Mesh> {
        let id = id.into_resource_id();
        self.meshes.get(id)
    }
}
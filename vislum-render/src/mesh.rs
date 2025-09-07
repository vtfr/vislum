use vislum_system::System;
use wgpu::util::DeviceExt;

use crate::types::RenderDevice;

use crate::resource::{Handle, IntoResourceId, ResourceStorage};

pub struct RenderMeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct RenderMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    buffer: wgpu::Buffer,
}

impl RenderMesh {
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

pub struct MeshManager {
    device: RenderDevice,
    meshes: ResourceStorage<RenderMesh>,
}

impl MeshManager {
    pub fn new(device: RenderDevice) -> Self {
        Self { device, meshes: ResourceStorage::new() }
    }

    pub fn create(&mut self, descriptor: RenderMeshDescriptor) -> Handle<RenderMesh> {
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.meshes.insert(RenderMesh {
            vertices: descriptor.vertices,
            indices: descriptor.indices,
            buffer,
        })
    }
    
    pub fn get(&self, id: impl IntoResourceId<RenderMesh>) -> Option<&RenderMesh> {
        let id = id.into_resource_id();
        self.meshes.get(id)
    }
}
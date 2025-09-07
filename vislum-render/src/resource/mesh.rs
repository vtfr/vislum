use bytemuck::{Pod, Zeroable};
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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
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
        &self.vertex_buffer
    }
    
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: 3 * (std::mem::size_of::<f32>() as wgpu::BufferAddress),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3,
            ],
        }
    }
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
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Buffer"),
            contents: &bytemuck::cast_slice(&descriptor.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: &bytemuck::cast_slice(&descriptor.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.meshes.insert(Mesh {
            vertices: descriptor.vertices,
            indices: descriptor.indices,
            vertex_buffer,
            index_buffer,
        })
    }
    
    pub fn get(&self, id: impl IntoResourceId<Mesh>) -> Option<&Mesh> {
        let id = id.into_resource_id();
        self.meshes.get(id)
    }
}
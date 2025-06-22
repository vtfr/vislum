/// Represents a Mesh as seen in the GPU.
pub struct MeshResource {
    pub buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
}

new_key_type! {
    pub struct MeshKey;
}

pub struct MeshManager {
    pub meshes: SlotMap<MeshKey, Mesh>,
}

pub struct Mesh {
    pub resource: MeshResource,
    pub vertex_buffer: wgpu::Buffer,
}
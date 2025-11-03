use std::sync::Arc;

use vislum_render_rhi::{
    device::Device,
    memory::MemoryAllocator,
    image::Image,
};

use crate::resource::{
    pool::{ResourceId, ResourcePool},
    texture::{Texture, TextureUploadTask, TextureCreateInfo},
    mesh::{Mesh, MeshUploadTask, Vertex},
};

pub mod pool;
pub mod texture;
pub mod mesh;

pub struct ResourceManager {
    device: Arc<Device>,
    allocator: Arc<MemoryAllocator>,
    textures: ResourcePool<Texture>,
    meshes: ResourcePool<Mesh>,
}

impl ResourceManager {
    pub fn new(device: Arc<Device>, allocator: Arc<MemoryAllocator>) -> Self {
        Self {
            device,
            allocator,
            textures: Default::default(),
            meshes: Default::default(),
        }
    }

    /// Creates a texture with data and returns the resource id and upload task.
    pub fn create_texture_with_data(
        &mut self,
        info: TextureCreateInfo,
        data: &[u8],
    ) -> (ResourceId<Texture>, TextureUploadTask) {
        let (texture, upload_task) = Texture::new_with_data(
            self.device.clone(),
            self.allocator.clone(),
            info,
            data,
        );
        let id = self.textures.insert(texture);
        (id, upload_task)
    }

    pub fn resolve_texture_image(&self, id: ResourceId<Texture>) -> Option<Arc<Image>> {
        self.textures.get(id).map(|texture| texture.image.clone())
    }

    /// Creates a mesh with data and returns the resource id and upload task.
    pub fn create_mesh(
        &mut self,
        vertices: impl IntoIterator<Item = Vertex>,
        indices: impl IntoIterator<Item = u16>,
    ) -> (ResourceId<Mesh>, MeshUploadTask) {
        let (mesh, upload_task) = Mesh::new(
            self.device.clone(),
            self.allocator.clone(),
            vertices,
            indices,
        );
        let id = self.meshes.insert(mesh);
        (id, upload_task)
    }

    pub fn get_mesh(&self, id: ResourceId<Mesh>) -> Option<&Mesh> {
        self.meshes.get(id)
    }
}

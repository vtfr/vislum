use std::sync::Arc;

use vulkano::{
    device::Device,
    image::{Image, view::ImageView},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
};

use crate::resource::{
    mesh::Mesh,
    pool::{ResourceId, ResourcePool},
    texture::{Texture, TextureDimensions, TextureFormat},
};

pub mod pool;
pub mod texture;
pub mod mesh;

pub struct ResourceManager {
    device: Arc<Device>,
    allocator: Arc<dyn MemoryAllocator>,
    textures: ResourcePool<Texture>,
    meshes: ResourcePool<Mesh>,
}

impl ResourceManager {
    pub fn new(device: Arc<Device>, allocator: Arc<dyn MemoryAllocator>) -> Self {
        Self {
            device,
            allocator,
            textures: Default::default(),
            meshes: Default::default(),
        }
    }

    pub fn create_texture(
        &mut self,
        format: TextureFormat,
        dimensions: TextureDimensions,
    ) -> ResourceId<Texture> {
        let image = vulkano::image::Image::new(
            self.allocator.clone(),
            vulkano::image::ImageCreateInfo {
                image_type: dimensions.into(),
                format: format.into(),
                extent: [1024, 1024, 1],
                mip_levels: 1,
                array_layers: 1,
                usage: vulkano::image::ImageUsage::TRANSFER_DST
                    | vulkano::image::ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();

        let view = ImageView::new_default(image.clone()).unwrap();

        self.textures.insert(Texture { image, view })
    }

    pub fn resolve_texture_image(&self, id: ResourceId<Texture>) -> Option<Arc<Image>> {
        self.textures.get(id).map(|texture| &texture.image).cloned()
    }

    pub fn create_mesh(
        &mut self,
        vertex_count: usize,
        index_count: usize,
    ) -> ResourceId<Mesh> {
        let mesh = Mesh::new(self.allocator.clone(), vertex_count, index_count);
        self.meshes.insert(mesh)
    }

    pub fn get_mesh(&self, id: ResourceId<Mesh>) -> Option<&Mesh> {
        self.meshes.get(id)
    }
}

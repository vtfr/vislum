use std::sync::Arc;

use ash::vk;
use vislum_render_rhi::{
    buffer::{Buffer, BufferCreateInfo},
    device::Device,
    image::{Image, ImageCreateInfo},
    memory::{MemoryAllocator, MemoryLocation},
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

    /// Creates a new texture and returns the resource id and the staging buffer
    /// with the data uploaded to it.
    /// 
    /// Caller is responsible for uploading the data to the staging buffer.
    pub fn create_texture(
        &mut self,
        format: TextureFormat,
        dimensions: TextureDimensions,
        data: &[u8],
    ) -> (ResourceId<Texture>, Arc<Buffer>) {
        self.create_texture_with_extent(format, dimensions, data, None)
    }

    pub fn create_texture_with_extent(
        &mut self,
        format: TextureFormat,
        dimensions: TextureDimensions,
        data: &[u8],
        extent: Option<[u32; 3]>,
    ) -> (ResourceId<Texture>, Arc<Buffer>) {
        // For 2D textures, calculate dimensions from data size if not provided
        let extent = match extent {
            Some(ext) => ext,
            None => match dimensions {
                TextureDimensions::D2 => {
                    // Assume square texture - calculate from data size
                    let pixel_count = data.len() / 4; // 4 bytes per RGBA pixel
                    let side = (pixel_count as f32).sqrt() as u32;
                    let side = side.max(1);
                    [side, side, 1]
                }
                TextureDimensions::D3 => {
                    // For 3D, use a reasonable default
                    [1024, 1024, 1024]
                }
            }
        };

        use vislum_render_rhi::image::ImageCreateInfo;
        use ash::vk;
        
        let image = Image::new(
            self.device.clone(),
            self.allocator.clone(),
            ImageCreateInfo {
                image_type: dimensions.into(),
                format: format.into(),
                extent: vk::Extent3D {
                    width: extent[0],
                    height: extent[1],
                    depth: extent[2],
                },
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                flags: vk::ImageCreateFlags::empty(),
            },
        );

        let id = self.textures.insert(Texture { image });

        // Create staging buffer with host-visible memory
        let staging = Buffer::new_with_location(
            self.device.clone(),
            self.allocator.clone(),
            BufferCreateInfo {
                size: data.len() as u64,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                flags: vk::BufferCreateFlags::empty(),
            },
            MemoryLocation::CpuToGpu,
        );

        // Write data to staging buffer
        unsafe {
            staging.write(data);
        }

        (id, staging)
    }

    pub fn resolve_texture_image(&self, id: ResourceId<Texture>) -> Option<Arc<Image>> {
        self.textures.get(id).map(|texture| &texture.image).cloned()
    }

    pub fn create_mesh(
        &mut self,
        vertex_count: usize,
        index_count: usize,
    ) -> ResourceId<Mesh> {
        let mesh = Mesh::new(self.device.clone(), self.allocator.clone(), vertex_count, index_count);
        self.meshes.insert(mesh)
    }

    pub fn get_mesh(&self, id: ResourceId<Mesh>) -> Option<&Mesh> {
        self.meshes.get(id)
    }
}

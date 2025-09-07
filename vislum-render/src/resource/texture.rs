use vislum_system::Resource;
use wgpu::util::DeviceExt;

use crate::cache::types::{RenderDevice, RenderQueue};
use crate::cache::storage::{Handle, IntoResourceId, ResourceStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba16Float,
    Rgba32Float,
}

impl Into<wgpu::TextureFormat> for TextureFormat {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

/// A descriptor for a texture.
/// 
/// The texture data is expected to be in the format of the texture format.
pub struct TextureDescriptor {
    pub format: TextureFormat,
    pub data: Option<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

pub struct Texture {
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub texture: wgpu::Texture,
    pub default_view: wgpu::TextureView,
}

impl Texture {
    /// Gets the default view of the texture.
    pub fn default_view(&self) -> &wgpu::TextureView {
        &self.default_view
    }

    /// Gets the texture of the texture.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

/// Manages textures.
#[derive(Resource)]
pub struct TextureManager {
    device: RenderDevice,
    queue: RenderQueue,
    textures: ResourceStorage<Texture>,
}

impl TextureManager {
    /// Creates a new texture manager.
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        Self { device, queue, textures: ResourceStorage::new() }
    }

    /// Creates a new texture.
    /// 
    /// Returns a owned handle to the texture.
    pub fn create(&mut self, descriptor: TextureDescriptor) -> Handle<Texture> {
        let wgpu_format = descriptor.format.into();
        let wgpu_texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: wgpu::Extent3d {
                width: descriptor.width,
                height: descriptor.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = match descriptor.data {
            Some(data) => {
                self.device.create_texture_with_data(
                    &*self.queue,
                    &wgpu_texture_descriptor,
                    wgpu::util::TextureDataOrder::LayerMajor,
                    &data,
                )
            }
            None => {
                self.device.create_texture(&wgpu_texture_descriptor)
            }
        };

        let default_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture View"),
            format: Some(wgpu_format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
            usage: None,
        });

        self.textures.insert(Texture {
            format: descriptor.format,
            width: descriptor.width,
            height: descriptor.height,
            texture,
            default_view,
        })
    }
    
    /// Gets a texture by its id.
    pub fn get(&self, id: impl IntoResourceId<Texture>) -> Option<&Texture> {
        let id = id.into_resource_id();
        self.textures.get(id)
    }
}
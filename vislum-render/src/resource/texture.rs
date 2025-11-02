use std::sync::Arc;

use ash::vk;
use vislum_render_rhi::image::Image;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8Srgb,
    Rgb8Unorm,
    Rgb8Srgb,
}

impl Into<vk::Format> for TextureFormat {
    fn into(self) -> vk::Format {
        match self {
            TextureFormat::Rgba8Unorm => vk::Format::R8G8B8A8_UNORM,
            TextureFormat::Rgba8Srgb => vk::Format::R8G8B8A8_SRGB,
            TextureFormat::Rgb8Unorm => vk::Format::R8G8B8_UNORM,
            TextureFormat::Rgb8Srgb => vk::Format::R8G8B8_SRGB,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDimensions {
    D2,
    D3,
}

impl Into<vk::ImageType> for TextureDimensions {
    fn into(self) -> vk::ImageType {
        match self {
            TextureDimensions::D2 => vk::ImageType::TYPE_2D,
            TextureDimensions::D3 => vk::ImageType::TYPE_3D,
        }
    }
}

pub struct Texture {
    pub(crate) image: Arc<Image>,
}
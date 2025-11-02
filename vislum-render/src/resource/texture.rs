use std::sync::Arc;

use vulkano::image::{Image, view::ImageView};



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8Srgb,
    Rgb8Unorm,
    Rgb8Srgb,
}

impl Into<vulkano::format::Format> for TextureFormat {
    fn into(self) -> vulkano::format::Format {
        use vulkano::format::Format as F;

        match self {
            TextureFormat::Rgba8Unorm => F::R8G8B8A8_UNORM,
            TextureFormat::Rgba8Srgb => F::R8G8B8A8_SRGB,
            TextureFormat::Rgb8Unorm => F::R8G8B8_UNORM,
            TextureFormat::Rgb8Srgb => F::R8G8B8_SRGB,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDimensions {
    D2,
    D3,
}

impl Into<vulkano::image::ImageType> for TextureDimensions {
    fn into(self) -> vulkano::image::ImageType {
        use vulkano::image::ImageType as IT;
        match self {
            TextureDimensions::D2 => IT::Dim2d,
            TextureDimensions::D3 => IT::Dim3d,
        }
    }
}

pub struct Texture {
    pub(crate) image: Arc<Image>,
    pub(crate) view: Arc<ImageView>,
}
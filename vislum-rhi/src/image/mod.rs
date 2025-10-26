pub mod color;
pub mod format;
pub mod image;
pub mod layout;
pub mod sampler;
pub mod view;

pub use color::RGBA;
pub use format::ImageFormat;
pub use image::{Image, ImageCreateInfo};
pub use layout::ImageLayout;
pub use sampler::{
    BorderColor, Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode,
};
pub use view::{ImageView, ImageViewCreateInfo};

use ash::vk;

pub type Viewport = vk::Viewport;
pub type Offset2D = vk::Offset2D;
pub type Extent2D = vk::Extent2D;
pub type Extent3D = vk::Extent3D;
pub type Rect2D = vk::Rect2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageDimensions {
    D1,
    D2,
    D3,
}

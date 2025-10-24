use std::sync::Arc;

use ash::vk;

use crate::{impl_vk_mapped_enum, rhi::device::Device};

#[derive(Debug, Clone, Copy)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl_vk_mapped_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ImageLayout: vk::ImageLayout {
        Undefined => UNDEFINED,
        ColorAttachment => COLOR_ATTACHMENT_OPTIMAL,
        DepthStencilAttachment => DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        DepthStencilReadOnly => DEPTH_STENCIL_READ_ONLY_OPTIMAL,
        ShaderReadOnly => SHADER_READ_ONLY_OPTIMAL,
        TransferSrcOptimal => TRANSFER_SRC_OPTIMAL,
        PresentSrc => PRESENT_SRC_KHR,
    }
}

impl_vk_mapped_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ImageDimension: vk::ImageType {
        D1 => TYPE_1D,
        D2 => TYPE_2D,
        D3 => TYPE_3D,
    }
}

impl_vk_mapped_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ImageFormat: vk::Format {
        Rgba8Unorm => R8G8B8A8_UNORM,
        Rgba8Srgb => R8G8B8A8_SRGB,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageDescription {
    pub extent: Extent2D,
    pub format: ImageFormat,
    pub dimensions: ImageDimension,
}

pub struct Image {
    device: Arc<Device>,
    image: vk::Image,
    description: ImageDescription,
}

impl Image {
    pub fn new(device: Arc<Device>, description: ImageDescription) -> Self {
        let create_info = vk::ImageCreateInfo::default()
            .image_type(description.dimensions.to_vk())
            .format(description.format.to_vk())
            .extent(vk::Extent3D::default()
                .width(description.extent.width)
                .height(description.extent.height)
                .depth(1))
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = unsafe {
            device.handle().create_image(&create_info, None).unwrap()
        };

        Self { device, image, description }
    }

    #[inline]
    pub fn format(&self) -> ImageFormat {
        self.description.format
    }

    #[inline]
    pub fn extent(&self) -> Extent2D {
        self.description.extent
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.description.extent.width
    }
    
    #[inline]
    pub fn height(&self) -> u32 {
        self.description.extent.height
    }

    #[inline]
    pub fn dimensions(&self) -> ImageDimension {
        self.description.dimensions
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_image(self.image, None);
        }
    }
}

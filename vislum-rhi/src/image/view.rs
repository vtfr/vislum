use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, image::ImageDimensions};

use super::{Image, ImageFormat};

#[derive(Debug, Clone)]
pub struct ImageViewCreateInfo {
    pub dimensions: ImageDimensions,
    pub format: Option<ImageFormat>,
}

impl Default for ImageViewCreateInfo {
    fn default() -> Self {
        Self {
            dimensions: ImageDimensions::D2,
            format: None,
        }
    }
}

pub struct ImageView {
    image: Arc<Image>,
    inner: vk::ImageView,
    dimensions: ImageDimensions,
    format: ImageFormat,
}

impl VkHandle for ImageView {
    type Handle = vk::ImageView;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.inner
    }
}

impl ImageView {
    pub fn new(image: Arc<Image>, create_info: ImageViewCreateInfo) -> Self {
        let format = create_info.format.unwrap_or(image.format());

        let view_type = match create_info.dimensions {
            ImageDimensions::D1 => vk::ImageViewType::TYPE_1D,
            ImageDimensions::D2 => vk::ImageViewType::TYPE_2D,
            ImageDimensions::D3 => vk::ImageViewType::TYPE_3D,
        };

        let vk_create_info = vk::ImageViewCreateInfo::default()
            .image(image.vk_handle())
            .view_type(view_type)
            .format(format.to_vk());

        let view = unsafe {
            image.device()
                .ash_handle()
                .create_image_view(&vk_create_info, None)
                .expect("Failed to create image view")
        };

        Self {
            image,
            inner: view,
            format,
            dimensions: create_info.dimensions,
        }
    }

    #[inline]
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    #[inline]
    pub fn dimensions(&self) -> ImageDimensions {
        self.dimensions
    }

    #[inline]
    pub fn format(&self) -> ImageFormat {
        self.format
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.image
                .device()
                .ash_handle()
                .destroy_image_view(self.inner, None);
        }
    }
}


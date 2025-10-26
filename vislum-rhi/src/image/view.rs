use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, image::ImageDimensions};

use super::{Image, ImageFormat};

#[derive(Debug, Clone)]
pub struct ImageViewCreateInfo {
    pub dimensions: ImageDimensions,
    pub format: ImageFormat,
}

/// The source of the image view.
enum ImageViewOwner {
    Image {
        image: Arc<Image>,
    },
    Swapchain,
}

pub struct ImageView {
    inner: vk::ImageView,
    dimensions: ImageDimensions,
    format: ImageFormat,
    owner: ImageViewOwner,
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
        let format = create_info.format;

        let view_type = match create_info.dimensions {
            ImageDimensions::D1 => vk::ImageViewType::TYPE_1D,
            ImageDimensions::D2 => vk::ImageViewType::TYPE_2D,
            ImageDimensions::D3 => vk::ImageViewType::TYPE_3D,
        };

        let vk_create_info = vk::ImageViewCreateInfo::default()
            .image(image.vk_handle())
            .view_type(view_type)
            .format(format.to_vk())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = unsafe {
            image.device()
                .ash_handle()
                .create_image_view(&vk_create_info, None)
                .expect("Failed to create image view")
        };

        Self {
            inner: view,
            format,
            dimensions: create_info.dimensions,
            owner: ImageViewOwner::Image { image },
        }
    }

    /// Creates a new image view from a swapchain.
    pub fn new_swapchain(image: &Image, create_info: ImageViewCreateInfo) -> Self {
        let format = create_info.format;

        let view_type = match create_info.dimensions {
            ImageDimensions::D1 => vk::ImageViewType::TYPE_1D,
            ImageDimensions::D2 => vk::ImageViewType::TYPE_2D,
            ImageDimensions::D3 => vk::ImageViewType::TYPE_3D,
        };

        let vk_create_info = vk::ImageViewCreateInfo::default()
            .image(image.vk_handle())
            .view_type(view_type)
            .format(format.to_vk())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let inner = unsafe {
            image.device()
                .ash_handle()
                .create_image_view(&vk_create_info, None)
                .expect("Failed to create image view")
        };
 
        Self {
            inner,
            format: create_info.format,
            dimensions: create_info.dimensions,
            owner: ImageViewOwner::Swapchain,
        }
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
            match &self.owner {
                ImageViewOwner::Image { image } => {
                    image.device()
                        .ash_handle()
                        .destroy_image_view(self.inner, None);
                }
                ImageViewOwner::Swapchain => {
                    // No need to destroy the image view, it's owned by the swapchain
                    // and will be destroyed when the swapchain is destroyed
                }
            }
        }
    }
}


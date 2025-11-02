use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, VkHandle, device::Device, vk_enum};
use super::{Format, Image, ImageType};

vk_enum! {
    #[derive(Default)]
    pub enum ImageViewType: vk::ImageViewType {
        /// A one-dimensional image.
        D1 => TYPE_1D,
        /// A two-dimensional image.
        #[default]
        D2 => TYPE_2D,
        /// A three-dimensional image.
        D3 => TYPE_3D,
    }
}

impl From<ImageType> for ImageViewType {
    fn from(image_type: ImageType) -> Self {
        match image_type {
            ImageType::D1 => ImageViewType::D1,
            ImageType::D2 => ImageViewType::D2,
            ImageType::D3 => ImageViewType::D3,
        }
    }
}

pub struct ImageViewCreateInfo {
    pub image: Arc<Image>,
    pub view_type: vk::ImageViewType,
    pub format: Format,
    pub components: vk::ComponentMapping,
    pub subresource_range: vk::ImageSubresourceRange,
}

pub struct ImageView {
    device: Arc<Device>,
    image_view: DebugWrapper<vk::ImageView>,
}

impl ImageView {
    pub fn new(device: Arc<Device>, create_info: ImageViewCreateInfo) -> Arc<Self> {
        let vk_create_info = vk::ImageViewCreateInfo::default()
            .image(create_info.image.vk_handle())
            .view_type(create_info.view_type)
            .format(create_info.format.to_vk())
            .components(create_info.components)
            .subresource_range(create_info.subresource_range);

        let image_view = unsafe {
            device.ash_handle().create_image_view(&vk_create_info, None).unwrap()
        };

        Arc::new(Self {
            device,
            image_view: DebugWrapper(image_view),
        })
    }
}

impl VkHandle for ImageView {
    type Handle = vk::ImageView;

    fn vk_handle(&self) -> Self::Handle {
        self.image_view.0
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_image_view(self.image_view.0, None);
        }
    }
}


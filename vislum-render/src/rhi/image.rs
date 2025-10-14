use std::sync::Arc;
use ash::vk;
use super::device::Device;

/// An image view for accessing image data
#[derive(Debug)]
pub struct ImageView {
    device: Arc<Device>,
    image_view: vk::ImageView,
}

impl ImageView {
    /// Create a new image view
    pub fn new(
        device: Arc<Device>,
        image: vk::Image,
        format: vk::Format,
        aspect_mask: vk::ImageAspectFlags,
    ) -> Self {
        let create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let image_view = unsafe { device.vk().create_image_view(&create_info, None).unwrap() };

        Self { device, image_view }
    }

    #[inline]
    pub fn handle(&self) -> vk::ImageView {
        self.image_view
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_image_view(self.image_view, None);
        }
    }
}

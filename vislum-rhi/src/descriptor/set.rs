use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, buffer::Buffer, image::ImageView};

use super::{DescriptorPool, DescriptorSetLayout};

pub struct DescriptorSet {
    pub(super) pool: Arc<DescriptorPool>,
    pub(super) layout: Arc<DescriptorSetLayout>,
    pub(super) set: vk::DescriptorSet,
}

impl VkHandle for DescriptorSet {
    type Handle = vk::DescriptorSet;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.set
    }
}

impl DescriptorSet {
    #[inline]
    pub fn layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.layout
    }

    /// Writes a buffer to the descriptor set.
    ///
    /// # Safety
    /// We keep no state on the descriptor set, so we can't guarantee that the buffer is still valid
    /// when the descriptor set is used.
    ///
    /// It's up to the upper layer to keep track of the buffer's lifetime.
    pub fn write_buffer(&self, binding: u32, buffer: &Buffer, offset: u64, range: u64) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer.vk_handle())
            .offset(offset)
            .range(range);

        let buffer_infos = [buffer_info];

        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_infos);

        unsafe {
            self.pool
                .device
                .ash_handle()
                .update_descriptor_sets(&[write], &[]);
        }
    }

    /// Writes an image to the descriptor set.
    ///
    /// # Safety
    /// We keep no state on the descriptor set, so we can't guarantee that the image is still valid
    /// when the descriptor set is used.
    ///
    /// It's up to the upper layer to keep track of the image's lifetime.
    pub fn write_image(
        &self,
        binding: u32,
        image_view: &ImageView,
        sampler: vk::Sampler,
        layout: vk::ImageLayout,
    ) {
        let image_info = vk::DescriptorImageInfo::default()
            .image_view(image_view.vk_handle())
            .sampler(sampler)
            .image_layout(layout);

        let image_infos = [image_info];

        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_infos);

        unsafe {
            self.pool
                .device
                .ash_handle()
                .update_descriptor_sets(&[write], &[]);
        }
    }
}

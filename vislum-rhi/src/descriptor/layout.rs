use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, VkHandle, device::device::Device, vk_enum};

vk_enum! {
    pub enum DescriptorType: vk::DescriptorType {
        Sampler = SAMPLER,
        CombinedImageSampler = COMBINED_IMAGE_SAMPLER,
        SampledImage = SAMPLED_IMAGE,
        StorageImage = STORAGE_IMAGE,
        UniformTexelBuffer = UNIFORM_TEXEL_BUFFER,
        StorageTexelBuffer = STORAGE_TEXEL_BUFFER,
        UniformBuffer = UNIFORM_BUFFER,
        StorageBuffer = STORAGE_BUFFER,
        UniformBufferDynamic = UNIFORM_BUFFER_DYNAMIC,
        StorageBufferDynamic = STORAGE_BUFFER_DYNAMIC,
        InputAttachment = INPUT_ATTACHMENT,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: vk::ShaderStageFlags,
}

#[derive(Debug)]
pub struct DescriptorSetLayoutCreateInfo {
    pub bindings: Vec<DescriptorSetLayoutBinding>,
}

pub struct DescriptorSetLayout {
    device: Arc<Device>,
    layout: vk::DescriptorSetLayout,
    bindings: Vec<DescriptorSetLayoutBinding>,
}

impl VkHandle for DescriptorSetLayout {
    type Handle = vk::DescriptorSetLayout;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.layout
    }
}

impl DescriptorSetLayout {
    pub fn new(device: Arc<Device>, create_info: DescriptorSetLayoutCreateInfo) -> Arc<Self> {
        let vk_bindings: Vec<vk::DescriptorSetLayoutBinding> = create_info
            .bindings
            .iter()
            .map(|binding| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(binding.binding)
                    .descriptor_type(binding.descriptor_type.to_vk())
                    .descriptor_count(binding.descriptor_count)
                    .stage_flags(binding.stage_flags)
            })
            .collect();

        let vk_create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&vk_bindings);

        let layout = unsafe {
            device
                .ash_handle()
                .create_descriptor_set_layout(&vk_create_info, None)
                .expect("Failed to create descriptor set layout")
        };

        Arc::new(Self {
            device,
            layout,
            bindings: create_info.bindings.to_vec(),
        })
    }

    #[inline]
    pub fn bindings(&self) -> &[DescriptorSetLayoutBinding] {
        &self.bindings
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

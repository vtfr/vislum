use std::sync::Arc;

use ash::vk;
use smallvec::SmallVec;

use crate::{AshHandle, DebugWrapper, device::Device, vk_enum, vk_enum_flags};

vk_enum! {
    pub enum DescriptorType: vk::DescriptorType {
        Sampler => SAMPLER,
        CombinedImageSampler => COMBINED_IMAGE_SAMPLER,
        SampledImage => SAMPLED_IMAGE,
        StorageImage => STORAGE_IMAGE,
        UniformTexelBuffer => UNIFORM_TEXEL_BUFFER,
        StorageTexelBuffer => STORAGE_TEXEL_BUFFER,
        UniformBuffer => UNIFORM_BUFFER,
        StorageBuffer => STORAGE_BUFFER,
        UniformBufferDynamic => UNIFORM_BUFFER_DYNAMIC,
        StorageBufferDynamic => STORAGE_BUFFER_DYNAMIC,
        InputAttachment => INPUT_ATTACHMENT,
    }
}

vk_enum_flags! {
    /// Flags that can be used to create a descriptor pool.
    pub struct DescriptorPoolCreateFlags: vk::DescriptorPoolCreateFlags {
        /// The descriptor pool can be freed after all descriptor sets have been freed.
        FREE_DESCRIPTOR_SET => FREE_DESCRIPTOR_SET,

        /// Specifies that if descriptors in this binding are updated between
        /// when the descriptor set is bound in a command buffer and when that
        /// command buffer is submitted to a queue, then the submission will use
        /// the most recently set descriptors for this binding and the updates
        /// do not invalidate the command buffer.
        ///
        /// Descriptor bindings created with this flag are also partially exempt
        /// from the external synchronization requirement in vkUpdateDescriptorSets.
        ///
        /// Multiple descriptors with this flag set can be updated concurrently
        /// in different threads, though the same descriptor must not be updated
        /// concurrently by two threads. Descriptors with this flag set can be
        /// updated concurrently with the set being bound to a command buffer in
        /// another thread, but not concurrently with the set being reset or freed.
        ///
        /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorBindingFlagBits.html>
        UPDATE_AFTER_BIND => UPDATE_AFTER_BIND,
    }
}

/// Describes the number of descriptors that can be allocated from a pool.
#[derive(Default)]
pub struct DescriptorPoolSizes {
    pub sampler: u32,
    pub combined_image_sampler: u32,
    pub sampled_image: u32,
    pub storage_image: u32,
    pub uniform_texel_buffer: u32,
    pub storage_texel_buffer: u32,
    pub uniform_buffer: u32,
    pub storage_buffer: u32,
    pub uniform_buffer_dynamic: u32,
    pub storage_buffer_dynamic: u32,
    pub input_attachment: u32,
}

pub struct DescriptorPoolCreateInfo {
    /// The sizes of the descriptor pools.
    ///
    /// At least one descriptor must have a non-zero size.
    pub sizes: DescriptorPoolSizes,

    /// The maximum number of sets that can be allocated from the pool.
    pub max_sets: u32,
}

impl Default for DescriptorPoolCreateInfo {
    fn default() -> Self {
        Self {
            sizes: DescriptorPoolSizes {
                sampler: 1024,
                sampled_image: 1024,
                ..DescriptorPoolSizes::default()
            },
            max_sets: 1000,
        }
    }
}

pub struct RawDescriptorPool {
    device: Arc<Device>,
    pool: DebugWrapper<vk::DescriptorPool>,
}

impl RawDescriptorPool {
    pub fn new(device: Arc<Device>, create_info: DescriptorPoolCreateInfo) -> Arc<Self> {
        use DescriptorType as DT;

        let DescriptorPoolCreateInfo { sizes, max_sets } = create_info;

        let pool_sizes = [
            (DT::Sampler, sizes.sampler),
            (DT::SampledImage, sizes.sampled_image),
            (DT::CombinedImageSampler, sizes.combined_image_sampler),
            (DT::StorageImage, sizes.storage_image),
            (DT::UniformTexelBuffer, sizes.uniform_texel_buffer),
            (DT::StorageTexelBuffer, sizes.storage_texel_buffer),
            (DT::UniformBuffer, sizes.uniform_buffer),
            (DT::StorageBuffer, sizes.storage_buffer),
            (DT::UniformBufferDynamic, sizes.uniform_buffer_dynamic),
            (DT::StorageBufferDynamic, sizes.storage_buffer_dynamic),
            (DT::InputAttachment, sizes.input_attachment),
        ]
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(ty, count)| {
            vk::DescriptorPoolSize::default()
                .ty(ty.to_vk())
                .descriptor_count(count)
        })
        .collect::<SmallVec<[vk::DescriptorPoolSize; 8]>>();

        let vk_create_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .pool_sizes(&*pool_sizes)
            .max_sets(max_sets);

        let pool = unsafe {
            device
                .ash_handle()
                .create_descriptor_pool(&vk_create_info, None)
                .unwrap()
        };

        Arc::new(Self {
            device,
            pool: DebugWrapper(pool),
        })
    }
}

pub struct DescriptorPool {
    device: Arc<Device>,
    create_info: DescriptorPoolCreateInfo,
    pools: Vec<RawDescriptorPool>,
}

impl DescriptorPool {
    pub fn new(device: Arc<Device>, create_info: DescriptorPoolCreateInfo) -> Arc<Self> {
        Arc::new(Self {
            device,
            create_info,
            pools: Default::default(),
        })
    }
}
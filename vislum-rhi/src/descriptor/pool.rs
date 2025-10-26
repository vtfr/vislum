use std::{collections::HashMap, sync::Arc};

use ash::vk;
use smallvec::SmallVec;

use crate::{AshHandle, VkHandle, descriptor::layout::DescriptorType, device::device::Device};

use super::{DescriptorSet, DescriptorSetLayout};

#[derive(Debug, Clone)]
pub struct DescriptorPoolCreateInfo {
    pub max_sets: u32,
    pub pool_sizes: HashMap<DescriptorType, u32>,
}

impl Default for DescriptorPoolCreateInfo {
    fn default() -> Self {
        Self {
            max_sets: 1000,
            pool_sizes: [
                (DescriptorType::UniformBuffer, 1000),
                (DescriptorType::StorageBuffer, 1000),
                (DescriptorType::UniformBufferDynamic, 1000),
                (DescriptorType::StorageBufferDynamic, 1000),
                (DescriptorType::InputAttachment, 1000),
            ].into_iter().collect()
        }
    }
}

pub struct DescriptorPool {
    pub(super) device: Arc<Device>,
    pool: vk::DescriptorPool,
}

impl VkHandle for DescriptorPool {
    type Handle = vk::DescriptorPool;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.pool
    }
}

impl DescriptorPool {
    pub fn new(device: Arc<Device>, create_info: DescriptorPoolCreateInfo) -> Arc<Self> {
        let vk_pool_sizes: Vec<vk::DescriptorPoolSize> = create_info
            .pool_sizes
            .iter()
            .map(|(descriptor_type, descriptor_count)| {
                vk::DescriptorPoolSize::default()
                    .ty(descriptor_type.to_vk())
                    .descriptor_count(*descriptor_count)
            })
            .collect();

        let vk_create_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(create_info.max_sets)
            .pool_sizes(&vk_pool_sizes);

        let pool = unsafe {
            device
                .ash_handle()
                .create_descriptor_pool(&vk_create_info, None)
                .expect("Failed to create descriptor pool")
        };

        Arc::new(Self { device, pool })
    }

    /// Allocates multiple descriptor sets from the pool.
    /// 
    /// # Safety
    /// This was designed around the assumption that we'll only have a few descriptor sets in the
    /// entire application, and these are allocated in the same thread as the descriptor pool was
    /// created.
    pub fn allocate(
        self: &Arc<Self>,
        layouts: impl ExactSizeIterator<Item = Arc<DescriptorSetLayout>>,
    ) -> impl ExactSizeIterator<Item = DescriptorSet> {
        let (vk_layouts, layouts): (SmallVec<[_; 8]>, SmallVec<[_; 8]>) = layouts
            .into_iter()
            .map(|layout| (layout.vk_handle(), layout.clone()))
            .unzip();

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(&vk_layouts);

        let sets = unsafe {
            self.device
                .ash_handle()
                .allocate_descriptor_sets(&alloc_info)
                .expect("Failed to allocate descriptor set")
        };

        layouts.into_iter()
            .zip(sets.into_iter())
            .map(|(layout, set)| DescriptorSet {
                pool: self.clone(),
                layout: layout.clone(),
                set,
            })
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ash_handle()
                .destroy_descriptor_pool(self.pool, None);
        }
    }
}


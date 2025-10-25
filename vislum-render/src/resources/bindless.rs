use std::sync::Arc;

use smallvec::SmallVec;
use vulkano::{
    descriptor_set::{
        layout::{
            DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding,
            DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType,
        },
        pool::{
            DescriptorPool, DescriptorPoolAlloc, DescriptorPoolCreateFlags,
            DescriptorPoolCreateInfo, DescriptorSetAllocateInfo,
        },
    },
    device::Device,
    shader::ShaderStages,
};

type DescriptorLayouts = SmallVec<[Arc<DescriptorSetLayout>; 4]>;
type DescriptorSets = SmallVec<[DescriptorPoolAlloc; 4]>;

/// A collection of descriptor set layouts for bindless resources.
pub struct BindlessDescriptorLayouts {
    /// The descriptor set layout for the image descriptors.
    image: Arc<DescriptorSetLayout>,

    /// The descriptor set layout for the sampler descriptors.
    sampler: Arc<DescriptorSetLayout>,
}

impl BindlessDescriptorLayouts {
    pub fn new(device: &Arc<Device>) -> Self {
        Self {
            image: Self::create_layout(device, DescriptorType::SampledImage),
            sampler: Self::create_layout(device, DescriptorType::Sampler),
        }
    }

    #[inline]
    pub fn image(&self) -> &Arc<DescriptorSetLayout> {
        &self.image
    }

    #[inline]
    pub fn sampler(&self) -> &Arc<DescriptorSetLayout> {
        &self.sampler
    }

    /// Returns an iterator over the cloned descriptor layouts.
    ///
    /// The order of the layouts is guaranteed to be:
    /// - Image descriptor layout
    /// - Sampler descriptor layout
    #[inline]
    pub fn iter_clone(&self) -> impl ExactSizeIterator<Item = Arc<DescriptorSetLayout>> {
        [self.image(), self.sampler()].into_iter().cloned()
    }

    fn create_layout(
        device: &Arc<Device>,
        descriptor_type: DescriptorType,
    ) -> Arc<DescriptorSetLayout> {
        let create_info = DescriptorSetLayoutCreateInfo {
            flags: DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL,
            bindings: [(
                0,
                DescriptorSetLayoutBinding {
                    binding_flags: DescriptorBindingFlags::UPDATE_AFTER_BIND
                        | DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
                    descriptor_count: 1000,
                    stages: ShaderStages::FRAGMENT,
                    immutable_samplers: vec![],
                    ..DescriptorSetLayoutBinding::descriptor_type(descriptor_type)
                },
            )]
            .into(),
            ..Default::default()
        };

        DescriptorSetLayout::new(device.clone(), create_info).unwrap()
    }
}

pub struct BindlessDescriptorSets {
    pool: DescriptorPool,
    image: DescriptorPoolAlloc,
    sampler: DescriptorPoolAlloc,
}

impl BindlessDescriptorSets {
    pub fn new(device: Arc<Device>, layouts: &BindlessDescriptorLayouts) -> Self {
        let pool_sizes = [
            (DescriptorType::SampledImage, 1u32),
            (DescriptorType::Sampler, 1u32),
        ];

        let create_info = DescriptorPoolCreateInfo {
            flags: DescriptorPoolCreateFlags::UPDATE_AFTER_BIND,
            max_sets: 4,
            pool_sizes: pool_sizes.into_iter().collect(),
            ..Default::default()
        };

        let pool = DescriptorPool::new(device, create_info).unwrap();

        let allocate_infos = layouts.iter_clone().map(DescriptorSetAllocateInfo::new);

        let mut descriptor_sets = unsafe { pool.allocate_descriptor_sets(allocate_infos) }.unwrap();

        let image = descriptor_sets.next().unwrap();
        let sampler = descriptor_sets.next().unwrap();

        Self {
            pool,
            image,
            sampler,
        }
    }
}

pub struct BindlessTable {
    device: Arc<Device>,
    layouts: BindlessDescriptorLayouts,
    sets: BindlessDescriptorSets,
}

impl BindlessTable {
    pub fn new(device: Arc<Device>) -> Self {
        let layouts = BindlessDescriptorLayouts::new(&device);
        let sets = BindlessDescriptorSets::new(device.clone(), &layouts);

        Self {
            device,
            layouts,
            sets,
        }
    }

    /// Returns a the descriptor layouts for the bindless table.
    #[inline]
    pub fn layouts(&self) -> &BindlessDescriptorLayouts {
        &self.layouts
    }

    /// Returns a the descriptor sets for the bindless table.
    #[inline]
    pub fn sets(&self) -> &BindlessDescriptorSets {
        &self.sets
    }
}

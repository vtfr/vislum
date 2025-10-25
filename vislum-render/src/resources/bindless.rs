use std::sync::Arc;

use vulkano::{
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::DescriptorSetAllocator, layout::{
            DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding,
            DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType,
        }
    },
    device::Device,
    shader::ShaderStages,
};

/// The type of a bindless resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindlessResourceType {
    Texture,
    Sampler,
}

/// A table for a single resource type.
pub struct BindlessResourceTable {
    resource_type: BindlessResourceType,
    descriptor_set: Arc<DescriptorSet>,
    next_index: u32,
}

pub struct BindlessTable {
    device: Arc<Device>,
    allocator: Arc<dyn DescriptorSetAllocator>,
    image_ds: Arc<DescriptorSet>,
    sampler_ds: Arc<DescriptorSet>,
}

impl BindlessTable {
    pub fn new(device: Arc<Device>, allocator: Arc<dyn DescriptorSetAllocator>) -> Self {
        let image_layout = Self::create_layout(&device, DescriptorType::SampledImage);
        let sampler_layout = Self::create_layout(&device, DescriptorType::Sampler);

        let image_ds = Self::create_descriptor_set(&allocator, image_layout);
        let sampler_ds = Self::create_descriptor_set(&allocator, sampler_layout);

        Self {
            device,
            allocator,
            image_ds,
            sampler_ds,
        }
    }

    #[inline]
    pub fn image_descriptor_layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.image_ds.layout()
    }

    #[inline]
    pub fn sampler_descriptor_layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.sampler_ds.layout()
    }

    /// Returns the image descriptor set.
    #[inline]
    pub fn image_descriptor_set(&self) -> &Arc<DescriptorSet> {
        &self.image_ds
    }

    /// Returns the sampler descriptor set.
    #[inline]
    pub fn sampler_descriptor_set(&self) -> &Arc<DescriptorSet> {
        &self.sampler_ds
    }

    fn create_layout(
        device: &Arc<Device>,
        descriptor_type: DescriptorType,
    ) -> Arc<DescriptorSetLayout> {
        let create_info = DescriptorSetLayoutCreateInfo {
            flags: DescriptorSetLayoutCreateFlags::empty(),
            bindings: [(
                0,
                DescriptorSetLayoutBinding {
                    binding_flags: DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
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

    fn create_descriptor_set(
        allocator: &Arc<dyn DescriptorSetAllocator>,
        layout: Arc<DescriptorSetLayout>,
    ) -> Arc<DescriptorSet> {
        DescriptorSet::new_variable(
            allocator.clone(),
            layout.clone(),
            1000,
            std::iter::empty(),
            std::iter::empty(),
        )
        .unwrap()
    }
}

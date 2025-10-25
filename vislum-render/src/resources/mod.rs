use std::{sync::Arc, u32, vec};

use vulkano::{descriptor_set::{DescriptorSet, allocator::DescriptorSetAllocator, layout::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType}, pool::DescriptorPool}, device::Device, shader::ShaderStages};

use crate::resources::{bindless::BindlessTable, id::ResourceStorage, texture::Texture};

pub mod id;
pub mod texture;
pub mod bindless;

pub struct ResourceManager {
    device: Arc<Device>,
    textures: ResourceStorage<Texture>,
    bindless_table: BindlessTable,
}

impl ResourceManager {
    pub fn new(
        device: Arc<Device>, 
        allocator: Arc<dyn DescriptorSetAllocator>,
        descriptor_pool: &DescriptorPool,
    ) -> Self {
        let bindless_table = BindlessTable::new(device.clone());

        Self {
            device,
            textures: ResourceStorage::default(),
            bindless_table,
        }
    }
}
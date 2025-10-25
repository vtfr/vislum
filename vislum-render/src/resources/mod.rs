use std::{fmt::Debug, sync::Arc, u32, vec};

use vulkano::{descriptor_set::{DescriptorSet, allocator::DescriptorSetAllocator, layout::{DescriptorBindingFlags, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType}, pool::DescriptorPool}, device::Device, image::ImageCreateInfo, memory::allocator::{AllocationCreateInfo, MemoryAllocator}, shader::ShaderStages};

use crate::resources::{bindless::BindlessTable, id::{ErasedResourceId, ResourceStorage}, texture::{Texture, TextureDescription, TextureHandle}};

pub mod id;
pub mod texture;
pub mod bindless;

pub struct ResourceManager {
    device: Arc<Device>,

    /// The memory allocator
    allocator: Arc<dyn MemoryAllocator>,

    /// A channel to send notifications when a resource is dropped.
    resource_drop_tx: crossbeam_channel::Sender<ErasedResourceId>,
    /// A channel to receive notifications when a resource is dropped.
    resource_drop_rx: crossbeam_channel::Receiver<ErasedResourceId>,

    /// The storage for the textures.
    textures: ResourceStorage<Texture>,

    /// The bindless table for the resources.
    bindless_table: BindlessTable,
}

impl Debug for ResourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceManager")
    }
}

impl ResourceManager {
    pub fn new(
        device: Arc<Device>, 
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
        memory_allocator: Arc<dyn MemoryAllocator>,
    ) -> Self {
        let bindless_table = BindlessTable::new(device.clone(), descriptor_set_allocator);

        let (resource_drop_tx, resource_drop_rx) = crossbeam_channel::unbounded::<ErasedResourceId>();

        Self {
            device,
            allocator: memory_allocator,
            resource_drop_rx,
            resource_drop_tx,
            textures: ResourceStorage::default(),
            bindless_table,
        }
    }

    pub fn create_texture(&mut self, description: TextureDescription) -> TextureHandle {
        let create_info = ImageCreateInfo {
            flags: vulkano::image::ImageCreateFlags::empty(),
            image_type: vulkano::image::ImageType::Dim2d,
            format: vulkano::format::Format::R8G8B8A8_SNORM,
            view_formats: vec![
                vulkano::format::Format::R8G8B8A8_SNORM,
            ],
            extent: [
                description.dimensions.width, 
                description.dimensions.height, 
                1,
            ],
            usage: vulkano::image::ImageUsage::SAMPLED | vulkano::image::ImageUsage::TRANSFER_DST,

            ..Default::default()
        };

        let image = vulkano::image::Image::new(
            self.allocator.clone(),
            create_info,
            AllocationCreateInfo {
                allocate_preference: vulkano::memory::allocator::MemoryAllocatePreference::AlwaysAllocate,
                ..Default::default()
            },
        ).unwrap();

        let default_view = vulkano::image::view::ImageView::new_default(image.clone()).unwrap();

        let texture = Texture {
            description,
            image,
            default_view,
        };

        let id = self.textures.insert(texture);

        TextureHandle::new(id, description, self.resource_drop_tx.clone())
    }
}
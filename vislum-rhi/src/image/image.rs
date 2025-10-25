use std::sync::Arc;

use ash::vk;

use crate::{
    AshHandle, VkHandle, device::device::Device, image::{Extent3D, ImageDimensions}, memory::allocator::{AllocationDescription, MemoryAllocation, MemoryAllocator, MemoryLocation}
};

use super::{ImageFormat, ImageLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageCreateInfo {
    pub dimensions: ImageDimensions,
    pub extent: Extent3D,
    pub format: ImageFormat,
    pub usage: vk::ImageUsageFlags,
}

impl Default for ImageCreateInfo {
    fn default() -> Self {
        Self {
            dimensions: ImageDimensions::D2,
            extent: Extent3D {
                width: 1,
                height: 1,
                depth: 0,
            },
            format: ImageFormat::R8G8B8A8Unorm,
            usage: vk::ImageUsageFlags::SAMPLED,
        }
    }
}

pub struct Image {
    device: Arc<Device>,
    image: vk::Image,
    allocation: MemoryAllocation,
    dimensions: ImageDimensions,
    extent: Extent3D,
    format: ImageFormat,
}

impl VkHandle for Image {
    type Handle = vk::Image;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.image
    }
}

impl Image {
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<MemoryAllocator>,
        create_info: ImageCreateInfo,
    ) -> Arc<Self> {
        let image_type = match create_info.dimensions {
            ImageDimensions::D1 => vk::ImageType::TYPE_1D,
            ImageDimensions::D2 => vk::ImageType::TYPE_2D,
            ImageDimensions::D3 => vk::ImageType::TYPE_3D,
        };

        let vk_create_info = vk::ImageCreateInfo::default()
            .image_type(image_type)
            .format(create_info.format.to_vk())
            .extent(create_info.extent)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(create_info.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(ImageLayout::Undefined.to_vk());

        let image = unsafe {
            device
                .ash_handle()
                .create_image(&vk_create_info, None)
                .expect("Failed to create image")
        };

        // Allocate memory for the image
        let requirements = unsafe { device.ash_handle().get_image_memory_requirements(image) };

        let allocation = allocator
            .allocate(AllocationDescription {
                name: Some("Image"),
                requirements,
                location: MemoryLocation::GpuOnly,
            })
            .expect("Failed to allocate memory for image");

        // Bind the memory to the image
        unsafe {
            device
                .ash_handle()
                .bind_image_memory(image, allocation.memory(), allocation.offset())
                .expect("Failed to bind image memory");
        }

        Arc::new(Self {
            device,
            image,
            allocation,
            dimensions: create_info.dimensions,
            extent: create_info.extent,
            format: create_info.format,
        })
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    #[inline]
    pub fn dimensions(&self) -> ImageDimensions {
        self.dimensions
    }

    #[inline]
    pub fn extent(&self) -> Extent3D {
        self.extent
    }

    #[inline]
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    #[inline]
    pub fn allocation(&self) -> &MemoryAllocation {
        &self.allocation
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.ash_handle().destroy_image(self.image, None);
        }
    }
}

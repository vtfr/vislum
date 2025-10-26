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

enum ImageOwner {
    Owned {
        allocation: MemoryAllocation,
    },
    Swapchain,
}

pub struct Image {
    device: Arc<Device>,
    image: vk::Image,
    owner: ImageOwner,
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
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
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
            owner: ImageOwner::Owned {
                allocation,
            },
            dimensions: create_info.dimensions,
            extent: create_info.extent,
            format: create_info.format,
        })
    }

    /// Creates a new image from a swapchain.
    pub fn from_swapchain(
        device: Arc<Device>,
        image: vk::Image,
        create_info: ImageCreateInfo,
    ) -> Self {
        Self {
            device,
            image,
            owner: ImageOwner::Swapchain,
            dimensions: create_info.dimensions,
            extent: create_info.extent,
            format: create_info.format,
        }
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
    pub fn allocation(&self) -> Option<&MemoryAllocation> {
        match &self.owner {
            ImageOwner::Owned { allocation } => Some(allocation),
            ImageOwner::Swapchain => None,
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            match &mut self.owner {
                ImageOwner::Owned { allocation } => {
                    self.device.ash_handle().destroy_image(self.image, None);
                }
                ImageOwner::Swapchain => {
                    // No need to destroy the image, it's owned by the swapchain
                    // and will be destroyed when the swapchain is destroyed
                }
            }
        }
    }
}

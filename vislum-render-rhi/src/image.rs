use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, VkHandle, device::Device, memory::{MemoryAllocation, MemoryAllocator}};

/// The owner of an image.
enum ImageOwner {
    /// The image is owned by the user.
    User {
        memory: MemoryAllocation,
    },
    /// The image is owned by the swapchain.
    Swapchain, 
}

pub struct ImageCreateInfo {
    pub image_type: vk::ImageType,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: vk::SampleCountFlags,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub flags: vk::ImageCreateFlags,
}

impl Default for ImageCreateInfo {
    fn default() -> Self {
        Self {
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_UNORM,
            extent: vk::Extent3D {
                width: 1024,
                height: 1024,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::empty(),
            flags: vk::ImageCreateFlags::empty(),
        }
    }
}

pub struct Image {
    device: Arc<Device>,
    image: DebugWrapper<vk::Image>,
    owner: ImageOwner,
}

impl Image {
    /// Creates a new image from the provided create info with allocated memory.
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<MemoryAllocator>,
        create_info: ImageCreateInfo,
    ) -> Self {
        let vk_create_info = vk::ImageCreateInfo::default()
            .image_type(create_info.image_type)
            .format(create_info.format)
            .extent(create_info.extent)
            .mip_levels(create_info.mip_levels)
            .array_layers(create_info.array_layers)
            .samples(create_info.samples)
            .tiling(create_info.tiling)
            .usage(create_info.usage)
            .flags(create_info.flags);

        let image = unsafe {
            device.ash_handle().create_image(&vk_create_info, None).unwrap()
        };

        // Get memory requirements for the image
        let memory_requirements = unsafe {
            device.ash_handle().get_image_memory_requirements(image)
        };

        // Allocate memory for the image
        let memory = allocator.allocate(
            memory_requirements,
            gpu_allocator::MemoryLocation::GpuOnly,
        );

        // Bind memory to the image
        unsafe {
            device
                .ash_handle()
                .bind_image_memory(image, memory.memory(), memory.offset())
                .unwrap();
        }

        Self {
            device,
            image: DebugWrapper(image),
            owner: ImageOwner::User {
                memory,
            },
        }
    }

    /// Creates a new image from a raw Vulkan handle (e.g., from a swapchain).
    pub fn from_raw(device: Arc<Device>, image: vk::Image) -> Self {
        Self {
            device,
            image: DebugWrapper(image),
            owner: ImageOwner::Swapchain,
        }
    }

    /// Returns the device associated with the image.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl VkHandle for Image {
    type Handle = vk::Image;

    fn vk_handle(&self) -> Self::Handle {
        self.image.0
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        // Only destroy the image if we own it (not if it came from a swapchain)
        match &self.owner {
            ImageOwner::User { .. } => {
                unsafe {
                    self.device.ash_handle().destroy_image(self.image.0, None);
                }
            }
            ImageOwner::Swapchain => {
                // Don't destroy swapchain images as they're managed by the swapchain
            }
        }
    }
}
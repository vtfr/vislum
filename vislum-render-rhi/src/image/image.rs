use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, VkHandle, device::Device, memory::{MemoryAllocation, MemoryAllocator, MemoryLocation}, swapchain::Swapchain, vk_enum, vk_enum_flags};
use super::{Extent3D, Format};

vk_enum! {
    #[derive(Default)]
    pub enum ImageType: ash::vk::ImageType {
        /// A one-dimensional image.
        D1 => TYPE_1D,
        /// A two-dimensional image.
        #[default]
        D2 => TYPE_2D,
        /// A three-dimensional image.
        D3 => TYPE_3D,
    }
}

vk_enum_flags! {
    pub struct ImageUsage: ash::vk::ImageUsageFlags {
        COLOR_ATTACHMENT => COLOR_ATTACHMENT,
        TRANSFER_DST => TRANSFER_DST,
        TRANSFER_SRC => TRANSFER_SRC,
        SAMPLED => SAMPLED,
    }
}

/// The owner of an image.
enum ImageStorage {
    /// The image is owned by the user.
    User {
        #[allow(dead_code)]
        memory: MemoryAllocation,
    },
    /// The image was created from a swapchain image.
    Swapchain {
        #[allow(dead_code)]
        swapchain: Arc<Swapchain>,
    }
}

pub struct ImageCreateInfo {
    pub dimensions: ImageType,
    pub format: Format,
    pub extent: Extent3D,
    pub mip_levels: u32,
    pub array_layers: u32,
    // pub samples: vk::SampleCountFlags,
    // pub tiling: vk::ImageTiling,
    pub usage: ImageUsage,
}

impl Default for ImageCreateInfo {
    fn default() -> Self {
        Self {
            dimensions: ImageType::D2,
            format: Format::Rgba8Unorm,
            extent: Extent3D::default(),
            mip_levels: 1,
            array_layers: 1,
            usage: ImageUsage::empty(),
        }
    }
}

pub struct Image {
    device: Arc<Device>,
    image: DebugWrapper<vk::Image>,
    owner: ImageStorage,
}

impl Image {
    /// Creates a new image from the provided create info with allocated memory.
    pub fn new(
        device: Arc<Device>,
        allocator: Arc<MemoryAllocator>,
        create_info: ImageCreateInfo,
        memory_location: MemoryLocation,
    ) -> Arc<Self> {
        let vk_create_info = vk::ImageCreateInfo::default()
            .image_type(create_info.dimensions.to_vk())
            .format(create_info.format.to_vk())
            .extent(create_info.extent.to_vk())
            .mip_levels(create_info.mip_levels)
            .array_layers(create_info.array_layers)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(create_info.usage.to_vk());

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
            memory_location,
        );

        // Bind memory to the image
        unsafe {
            device
                .ash_handle()
                .bind_image_memory(image, memory.memory(), memory.offset())
                .unwrap();
        }

        Arc::new(Self {
            device,
            image: DebugWrapper(image),
            owner: ImageStorage::User {
                memory,
            },
        })
    }

    pub(crate) fn from_swapchain_image(swapchain: Arc<Swapchain>, swapchain_image: vk::Image) -> Arc<Self> {
        Arc::new(Self {
            device: swapchain.device().clone(),
            image: DebugWrapper(swapchain_image),
            owner: ImageStorage::Swapchain {
                swapchain,
            },
        })
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
            ImageStorage::User { .. } => {
                unsafe {
                    self.device.ash_handle().destroy_image(self.image.0, None);
                }
            }
            ImageStorage::Swapchain { .. } => {
                // Don't destroy swapchain images as they're managed by the swapchain
            }
        }
    }
}


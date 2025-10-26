use std::sync::Arc;

use ash::vk;
use smallvec::SmallVec;

use crate::{
    Error, VkHandle, WithContext, device::device::Device, image::{
        Extent2D, Extent3D, Image, ImageCreateInfo, ImageDimensions, ImageFormat, ImageView,
        ImageViewCreateInfo,
    }, surface::Surface
};

pub type PresentMode = vk::PresentModeKHR;

pub struct SwapchainCreateInfo {
    pub extent: Extent2D,
    pub format: ImageFormat,
    pub present_mode: PresentMode,
}

impl Default for SwapchainCreateInfo {
    fn default() -> Self {
        Self {
            extent: Extent2D {
                width: 1920,
                height: 1080,
            },
            format: ImageFormat::B8G8R8A8Srgb,
            present_mode: vk::PresentModeKHR::FIFO,
        }
    }
}

pub struct Swapchain {
    device: Arc<Device>,
    swapchain: vk::SwapchainKHR,
    images: SmallVec<[Image; 3]>,
    views: SmallVec<[ImageView; 3]>,
    format: ImageFormat,
    extent: Extent2D,
}

impl VkHandle for Swapchain {
    type Handle = vk::SwapchainKHR;

    #[inline]
    fn vk_handle(&self) -> Self::Handle {
        self.swapchain
    }
}

impl Swapchain {
    pub fn new(
        device: Arc<Device>,
        surface: &Arc<Surface>,
        create_info: SwapchainCreateInfo,
    ) -> Result<Arc<Self>, Error> {
        let vk_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.vk_handle())
            .min_image_count(3)
            .image_format(create_info.format.to_vk())
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(create_info.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(create_info.present_mode)
            .clipped(true);

        let swapchain = unsafe {
            device
                .ash_khr_swapchain()
                .create_swapchain(&vk_create_info, None)
                .with_context("failed to create swapchain")?
        };

        let images = unsafe {
            device
                .ash_khr_swapchain()
                .get_swapchain_images(swapchain)
                .with_context("failed to get swapchain images")?
        };

        let images: SmallVec<[Image; 3]> = images
            .into_iter()
            .map(|image| {
                Image::from_swapchain(
                    device.clone(),
                    image,
                    ImageCreateInfo {
                        dimensions: ImageDimensions::D2,
                        extent: Extent3D {
                            width: create_info.extent.width,
                            height: create_info.extent.height,
                            depth: 1,
                        },
                        format: create_info.format,
                        usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                    },
                )
            })
            .collect();

        let views = images
            .iter()
            .map(|image| {
                ImageView::new_swapchain(
                    image,
                    ImageViewCreateInfo {
                        dimensions: ImageDimensions::D2,
                        format: create_info.format,
                    },
                )
            })
            .collect();

        Ok(Arc::new(Self {
            device,
            swapchain,
            images,
            views,
            format: create_info.format,
            extent: create_info.extent,
        }))
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Acquires the next image from the swapchain.
    ///
    /// Returns the index of the image if it was acquired successfully, otherwise `None`.
    pub fn acquire_next_image(&self, semaphore: vk::Semaphore) -> Option<u32> {
        unsafe {
            self.device
                .ash_khr_swapchain()
                .acquire_next_image(self.swapchain, u64::MAX, semaphore, vk::Fence::null())
                .ok()
                .map(|(index, _)| index)
        }
    }

    pub fn present(&self, queue: vk::Queue, image_index: u32, wait_semaphore: vk::Semaphore) {
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let wait_semaphores = [wait_semaphore];

        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .wait_semaphores(&wait_semaphores);

        unsafe {
            let _ = self
                .device
                .ash_khr_swapchain()
                .queue_present(queue, &present_info);
        }
    }

    #[inline]
    pub fn images(&self) -> &[Image] {
        &self.images
    }

    #[inline]
    pub fn views(&self) -> &[ImageView] {
        &self.views
    }

    #[inline]
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    #[inline]
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device
                .ash_khr_swapchain()
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

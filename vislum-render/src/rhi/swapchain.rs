use std::{ffi::CStr, sync::Arc};

use ash::vk;

use super::{device::Device, surface::Surface};

#[derive(Debug, thiserror::Error)]
pub enum SwapchainError {
    #[error("required swapchain extension not enabled: {0}")]
    ExtensionNotEnabled(&'static str),
    #[error("vulkan error: {0}")]
    VulkanError(#[from] vk::Result),
}

impl SwapchainError {
    #[inline]
    pub(crate) const fn extension_not_enabled(extension: &'static CStr) -> Self {
        match extension.to_str() {
            Ok(extension) => Self::ExtensionNotEnabled(extension),
            Err(_) => unreachable!(),
        }
    }
}

/// Description for creating a swapchain
pub struct SwapchainDescription {
    /// The surface to present to
    pub surface: Arc<Surface>,
    /// The desired image format (if None, will pick the first available)
    pub format: Option<vk::Format>,
    /// The desired color space (if None, will pick the first available)
    pub color_space: Option<vk::ColorSpaceKHR>,
    /// The desired present mode (if None, will default to FIFO)
    pub present_mode: Option<vk::PresentModeKHR>,
    /// The desired number of images in the swapchain (if None, will use minimum + 1)
    pub image_count: Option<u32>,
    /// The width of the swapchain images
    pub width: u32,
    /// The height of the swapchain images
    pub height: u32,
}

/// A Vulkan swapchain for presenting rendered images to a surface
#[derive(Debug)]
pub struct Swapchain {
    device: Arc<Device>,
    surface: Arc<Surface>,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    format: vk::Format,
    color_space: vk::ColorSpaceKHR,
    extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
}

impl Swapchain {
    /// Create a new swapchain
    pub fn new(
        device: Arc<Device>,
        description: SwapchainDescription,
    ) -> Result<Arc<Self>, SwapchainError> {
        use ash::khr;

        let SwapchainDescription {
            surface,
            format,
            color_space,
            present_mode,
            image_count,
            width,
            height,
        } = description;

        // Check that the swapchain extension is enabled
        if device.fns().khr_swapchain().is_none() {
            return Err(SwapchainError::extension_not_enabled(khr::swapchain::NAME));
        }

        let physical_device = device.physical_device();
        
        // Query surface capabilities
        let surface_fns = device.instance().fns().khr_surface()
            .expect("khr_surface must be loaded to use swapchains");
        
        // Query surface capabilities
        let mut surface_capabilities = vk::SurfaceCapabilitiesKHR::default();
        unsafe {
            (surface_fns.get_physical_device_surface_capabilities_khr)(
                physical_device.handle(),
                surface.handle(),
                &mut surface_capabilities,
            ).result()?;
        }

        // Query surface formats
        let surface_formats = unsafe {
            crate::rhi::util::read_into_vec(|count, data| {
                (surface_fns.get_physical_device_surface_formats_khr)(
                    physical_device.handle(),
                    surface.handle(),
                    count,
                    data,
                )
            })?
        };

        // Query present modes
        let present_modes = unsafe {
            crate::rhi::util::read_into_vec(|count, data| {
                (surface_fns.get_physical_device_surface_present_modes_khr)(
                    physical_device.handle(),
                    surface.handle(),
                    count,
                    data,
                )
            })?
        };

        // Select surface format
        let (format, color_space) = if let (Some(format), Some(color_space)) = 
            (description.format, description.color_space) {
            // Verify the requested format is supported
            if !surface_formats.iter().any(|sf| sf.format == format && sf.color_space == color_space) {
                return Err(SwapchainError::VulkanError(vk::Result::ERROR_FORMAT_NOT_SUPPORTED));
            }
            (format, color_space)
        } else {
            // Pick the first available format
            let surface_format = surface_formats.first()
                .ok_or(vk::Result::ERROR_FORMAT_NOT_SUPPORTED)?;
            (surface_format.format, surface_format.color_space)
        };

        // Select present mode
        let present_mode = if let Some(mode) = description.present_mode {
            if !present_modes.contains(&mode) {
                return Err(SwapchainError::VulkanError(vk::Result::ERROR_FEATURE_NOT_PRESENT));
            }
            mode
        } else {
            // Default to FIFO (always supported)
            vk::PresentModeKHR::FIFO
        };

        // Select image count
        let image_count = if let Some(count) = description.image_count {
            count.clamp(
                surface_capabilities.min_image_count,
                if surface_capabilities.max_image_count > 0 {
                    surface_capabilities.max_image_count
                } else {
                    u32::MAX
                },
            )
        } else {
            // Use minimum + 1 for better performance
            let desired = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0 {
                desired.min(surface_capabilities.max_image_count)
            } else {
                desired
            }
        };

        // Determine extent
        let extent = if surface_capabilities.current_extent.width != u32::MAX {
            surface_capabilities.current_extent
        } else {
            vk::Extent2D {
                width: description.width.clamp(
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: description.height.clamp(
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            }
        };

        // Create swapchain
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.handle())
            .min_image_count(image_count)
            .image_format(format)
            .image_color_space(color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain_fns = device.fns().khr_swapchain().unwrap();
        let mut swapchain_khr = vk::SwapchainKHR::null();
        unsafe {
            (swapchain_fns.create_swapchain_khr)(
                device.handle(),
                &create_info,
                std::ptr::null(),
                &mut swapchain_khr,
            ).result()?;
        }

        // Get swapchain images
        let images = unsafe {
            crate::rhi::util::read_into_vec(|count, data| {
                (swapchain_fns.get_swapchain_images_khr)(
                    device.handle(),
                    swapchain_khr,
                    count,
                    data,
                )
            })?
        };

        Ok(Arc::new(Self {
            device,
            surface,
            swapchain: swapchain_khr,
            images,
            format,
            color_space,
            extent,
            present_mode,
        }))
    }

    #[inline]
    pub fn handle(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    #[inline]
    pub fn images(&self) -> &[vk::Image] {
        &self.images
    }

    #[inline]
    pub fn format(&self) -> vk::Format {
        self.format
    }

    #[inline]
    pub fn color_space(&self) -> vk::ColorSpaceKHR {
        self.color_space
    }

    #[inline]
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    #[inline]
    pub fn present_mode(&self) -> vk::PresentModeKHR {
        self.present_mode
    }

    #[inline]
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    #[inline]
    pub fn surface(&self) -> &Arc<Surface> {
        &self.surface
    }

    /// Acquire the next image from the swapchain.
    /// 
    /// Returns the index of the acquired image.
    pub fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: Option<vk::Semaphore>,
        fence: Option<vk::Fence>,
    ) -> Result<(u32, bool), vk::Result> {
        let swapchain_fns = self.device.fns().khr_swapchain().unwrap();
        
        let mut image_index = 0;
        let result = unsafe {
            (swapchain_fns.acquire_next_image_khr)(
                self.device.handle(),
                self.swapchain,
                timeout,
                semaphore.unwrap_or(vk::Semaphore::null()),
                fence.unwrap_or(vk::Fence::null()),
                &mut image_index,
            )
        };

        match result {
            vk::Result::SUCCESS => Ok((image_index, false)),
            vk::Result::SUBOPTIMAL_KHR => Ok((image_index, true)),
            _ => Err(result),
        }
    }

    /// Present the swapchain image to the surface.
    /// 
    /// Returns true if the swapchain is suboptimal.
    pub fn queue_present(
        &self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool, vk::Result> {
        let swapchain_fns = self.device.fns().khr_swapchain().unwrap();
        
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe {
            (swapchain_fns.queue_present_khr)(queue, &present_info)
        };

        match result {
            vk::Result::SUCCESS => Ok(false),
            vk::Result::SUBOPTIMAL_KHR => Ok(true),
            _ => Err(result),
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        // SAFETY: khr_swapchain MUST be loaded, otherwise the swapchain would not have been created
        let swapchain_fns = self.device.fns().khr_swapchain().unwrap();
        
        unsafe {
            (swapchain_fns.destroy_swapchain_khr)(
                self.device.handle(),
                self.swapchain,
                std::ptr::null(),
            );
        }
    }
}

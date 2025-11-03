use std::sync::Arc;

use ash::vk;

use crate::{AshHandle, DebugWrapper, VkHandle, device::Device, surface::Surface, image::{ImageFormat, Extent2D, Image, ImageUsage}, vk_enum};

vk_enum! {
    #[derive(Default)]
    pub enum PresentMode: vk::PresentModeKHR {
        #[default]
        FIFO => FIFO,
        MAILBOX => MAILBOX,
        IMMEDIATE => IMMEDIATE,
    }
}

pub struct SwapchainCreateInfo {
    /// Minimum number of images in the swapchain.
    /// Defaults to 2 if not specified.
    pub min_image_count: Option<u32>,
    /// Desired present mode. If not specified, FIFO will be used.
    pub present_mode: Option<PresentMode>,
    /// Desired image usage flags. If not specified, COLOR_ATTACHMENT will be used.
    pub image_usage: Option<ImageUsage>,
    /// Previous swapchain to replace (for resize operations).
    pub old_swapchain: Option<Arc<Swapchain>>,
}

pub struct Swapchain {
    device: Arc<Device>,
    swapchain: DebugWrapper<vk::SwapchainKHR>,
    swapchain_loader: ash::khr::swapchain::Device,
    surface: Arc<Surface>,
    image_format: ImageFormat,
    image_extent: Extent2D,
}

impl Swapchain {
    /// Creates a new swapchain from a surface.
    /// Most parameters are automatically derived from the surface capabilities.
    /// Returns the swapchain and its images.
    pub fn new(
        device: Arc<Device>,
        surface: Arc<Surface>,
        create_info: SwapchainCreateInfo,
    ) -> (Arc<Self>, Vec<Arc<Image>>) {
        let physical_device = device.physical_device();
        
        // Get surface capabilities and formats
        let capabilities = surface.get_capabilities(physical_device);
        let formats = surface.get_formats(physical_device);
        let present_modes = surface.get_present_modes(physical_device);

        // Choose format - prefer B8G8R8A8_UNORM with SRGB_NONLINEAR, otherwise first available
        let surface_format = formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or_else(|| formats[0]);
        
        let image_format = ImageFormat::from_vk(surface_format.format)
            .unwrap_or(ImageFormat::Rgba8Unorm);

        // Choose image count
        let mut image_count = create_info
            .min_image_count
            .unwrap_or(capabilities.min_image_count.max(2));
        if capabilities.max_image_count > 0 {
            image_count = image_count.min(capabilities.max_image_count);
        }

        // Choose extent
        let image_extent_vk = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: capabilities.min_image_extent.width.max(800).min(capabilities.max_image_extent.width),
                height: capabilities.min_image_extent.height.max(600).min(capabilities.max_image_extent.height),
            }
        };
        let image_extent = Extent2D::from_vk(image_extent_vk);

        // Choose present mode - prefer FIFO (guaranteed), or Mailbox (vsync), or first available
        let present_mode_vk = create_info.present_mode.map(|p| p.to_vk()).unwrap_or_else(|| {
            if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
                vk::PresentModeKHR::MAILBOX
            } else if present_modes.contains(&vk::PresentModeKHR::FIFO) {
                vk::PresentModeKHR::FIFO
            } else {
                present_modes[0]
            }
        });

        // Choose image usage - prefer COLOR_ATTACHMENT, but respect capabilities
        let requested_usage = create_info
            .image_usage
            .unwrap_or(ImageUsage::COLOR_ATTACHMENT);
        let image_usage_vk = requested_usage.to_vk() & capabilities.supported_usage_flags;

        // Use current transform from capabilities
        let pre_transform = capabilities.current_transform;

        // Choose composite alpha - prefer OPAQUE, fallback to first supported
        let composite_alpha = if capabilities
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::OPAQUE)
        {
            vk::CompositeAlphaFlagsKHR::OPAQUE
        } else {
            // Find first supported alpha mode
            [
                vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
                vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
                vk::CompositeAlphaFlagsKHR::INHERIT,
            ]
            .iter()
            .find(|mode| capabilities.supported_composite_alpha.contains(**mode))
            .copied()
            .unwrap_or(vk::CompositeAlphaFlagsKHR::OPAQUE)
        };

        let swapchain_loader = ash::khr::swapchain::Device::new(
            device.instance().ash_handle(),
            device.ash_handle(),
        );

        let old_swapchain = create_info
            .old_swapchain
            .as_ref()
            .map(|s| s.swapchain.0)
            .unwrap_or(vk::SwapchainKHR::null());

        let create_info_vk = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.vk_handle())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(image_extent_vk)
            .image_array_layers(1)
            .image_usage(image_usage_vk)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(composite_alpha)
            .present_mode(present_mode_vk)
            .clipped(true)
            .old_swapchain(old_swapchain);

        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&create_info_vk, None)
        }.unwrap();

        let images_vk = unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
        }.unwrap();

        let swapchain_arc = Arc::new(Self {
            device: device.clone(),
            swapchain: DebugWrapper(swapchain),
            swapchain_loader,
            surface,
            image_format,
            image_extent,
        });

        // Create Image wrappers for swapchain images
        let images: Vec<Arc<Image>> = images_vk.iter()
            .map(|&image_vk| Image::from_swapchain_image(swapchain_arc.clone(), image_vk))
            .collect();

        (swapchain_arc, images)
    }

    /// Acquires the next image from the swapchain.
    pub fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: Option<&crate::sync::Semaphore>,
        fence: Option<&crate::sync::Fence>,
    ) -> (u32, bool) {
        use crate::VkHandle;
        unsafe {
            self.swapchain_loader
                .acquire_next_image(
                    self.swapchain.0,
                    timeout,
                    semaphore.map(|s| s.vk_handle()).unwrap_or(vk::Semaphore::null()),
                    fence.map(|f| f.vk_handle()).unwrap_or(vk::Fence::null()),
                )
        }.unwrap()
    }

    /// Presents an image to the surface.
    pub fn present(
        &self,
        queue: &crate::queue::Queue,
        image_index: u32,
        wait_semaphores: &[&crate::sync::Semaphore],
    ) -> bool {
        use crate::VkHandle;
        let swapchain_handle = self.vk_handle();
        let queue_handle = queue.vk_handle();
        let semaphore_handles: Vec<_> = wait_semaphores.iter().map(|s| s.vk_handle()).collect();
        let swapchains = [swapchain_handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .wait_semaphores(&semaphore_handles);
        
        unsafe {
            self.swapchain_loader
                .queue_present(queue_handle, &present_info)
        }.unwrap()
    }


    /// Gets the image format.
    pub fn image_format(&self) -> ImageFormat {
        self.image_format
    }

    /// Gets the image extent.
    pub fn image_extent(&self) -> Extent2D {
        self.image_extent
    }

    /// Gets the device.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Gets the surface.
    pub fn surface(&self) -> &Arc<Surface> {
        &self.surface
    }
}

impl crate::VkHandle for Swapchain {
    type Handle = vk::SwapchainKHR;

    fn vk_handle(&self) -> Self::Handle {
        self.swapchain.0
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader.destroy_swapchain(self.swapchain.0, None);
        }
    }
}


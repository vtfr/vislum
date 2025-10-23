// use std::{ffi::CStr, sync::Arc};

// use ash::{khr, vk};

// use crate::rhi::{device::Device, surface::surface::Surface};

// /// Description for creating a swapchain
// pub struct SwapchainDescription {
//     /// The surface to present to.
//     pub surface: Arc<Surface>,
//     /// The width of the swapchain images
//     pub width: u32,
//     /// The height of the swapchain images
//     pub height: u32,
// }

// /// A Vulkan swapchain for presenting rendered images to a surface
// #[derive(Debug)]
// pub struct Swapchain {
//     device: Arc<Device>,
//     surface: Arc<Surface>,
//     swapchain: vk::SwapchainKHR,
//     images: Vec<vk::Image>,
//     format: vk::Format,
//     color_space: vk::ColorSpaceKHR,
//     extent: vk::Extent2D,
//     present_mode: vk::PresentModeKHR,
// }

// impl Swapchain {
//     /// Create a new swapchain
//     pub fn new(
//         device: Arc<Device>,
//         description: SwapchainDescription,
//     ) -> Result<Self, SwapchainError> {
//         use ash::khr;

//         let SwapchainDescription {
//             surface,
//             format,
//             color_space,
//             present_mode,
//             image_count,
//             width,
//             height,
//         } = description;

//         // Check that the swapchain extension is enabled
//         let khr_surface_instance = device
//             .instance()
//             .khr_surface_handle()
//             .ok_or(SwapchainError::extension_not_enabled(khr::surface::NAME))?;
//         let khr_swapchain_device = device
//             .khr_swapchain_device()
//             .ok_or(SwapchainError::extension_not_enabled(khr::swapchain::NAME))?;

//         let physical_device = device.physical_device();

//         // Query surface capabilities
//         let surface_capabilities = unsafe {
//             khr_surface_instance.get_physical_device_surface_capabilities(
//                 physical_device.handle(),
//                 surface.handle(),
//             )?
//         };

//         // Query surface formats
//         let surface_formats = unsafe {
//             khr_surface_instance
//                 .get_physical_device_surface_formats(physical_device.handle(), surface.handle())?
//         };

//         // Query present modes
//         let present_modes = unsafe {
//             khr_surface_instance.get_physical_device_surface_present_modes(
//                 physical_device.handle(),
//                 surface.handle(),
//             )?
//         };

//         // Select surface format
//         let (format, color_space) = if let (Some(format), Some(color_space)) =
//             (description.format, description.color_space)
//         {
//             // Verify the requested format is supported
//             if !surface_formats
//                 .iter()
//                 .any(|sf| sf.format == format && sf.color_space == color_space)
//             {
//                 return Err(SwapchainError::VulkanError(
//                     vk::Result::ERROR_FORMAT_NOT_SUPPORTED,
//                 ));
//             }
//             (format, color_space)
//         } else {
//             // Pick the first available format
//             let surface_format = surface_formats
//                 .first()
//                 .ok_or(vk::Result::ERROR_FORMAT_NOT_SUPPORTED)?;
//             (surface_format.format, surface_format.color_space)
//         };

//         // Select present mode
//         let present_mode = if let Some(mode) = description.present_mode {
//             if !present_modes.contains(&mode) {
//                 return Err(SwapchainError::VulkanError(
//                     vk::Result::ERROR_FEATURE_NOT_PRESENT,
//                 ));
//             }
//             mode
//         } else {
//             // Default to FIFO (always supported)
//             vk::PresentModeKHR::FIFO
//         };

//         // Select image count
//         let image_count = if let Some(count) = description.image_count {
//             count.clamp(
//                 surface_capabilities.min_image_count,
//                 if surface_capabilities.max_image_count > 0 {
//                     surface_capabilities.max_image_count
//                 } else {
//                     u32::MAX
//                 },
//             )
//         } else {
//             // Use minimum + 1 for better performance
//             let desired = surface_capabilities.min_image_count + 1;
//             if surface_capabilities.max_image_count > 0 {
//                 desired.min(surface_capabilities.max_image_count)
//             } else {
//                 desired
//             }
//         };

//         // Determine extent
//         let extent = if surface_capabilities.current_extent.width != u32::MAX {
//             surface_capabilities.current_extent
//         } else {
//             vk::Extent2D {
//                 width: description.width.clamp(
//                     surface_capabilities.min_image_extent.width,
//                     surface_capabilities.max_image_extent.width,
//                 ),
//                 height: description.height.clamp(
//                     surface_capabilities.min_image_extent.height,
//                     surface_capabilities.max_image_extent.height,
//                 ),
//             }
//         };

//         // Create swapchain
//         let create_info = vk::SwapchainCreateInfoKHR::default()
//             .surface(surface.handle())
//             .min_image_count(image_count)
//             .image_format(format)
//             .image_color_space(color_space)
//             .image_extent(extent)
//             .image_array_layers(1)
//             .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
//             .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
//             .pre_transform(surface_capabilities.current_transform)
//             .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
//             .present_mode(present_mode)
//             .clipped(true);

//         let swapchain_khr = unsafe { khr_swapchain_device.create_swapchain(&create_info, None)? };

//         // Get swapchain images
//         let images = unsafe { khr_swapchain_device.get_swapchain_images(swapchain_khr)? };

//         Ok(Self {
//             device,
//             surface,
//             swapchain: swapchain_khr,
//             images,
//             format,
//             color_space,
//             extent,
//             present_mode,
//         })
//     }

//     #[inline]
//     pub fn handle(&self) -> vk::SwapchainKHR {
//         self.swapchain
//     }

//     #[inline]
//     pub fn images(&self) -> &[vk::Image] {
//         &self.images
//     }

//     #[inline]
//     pub fn format(&self) -> vk::Format {
//         self.format
//     }

//     #[inline]
//     pub fn color_space(&self) -> vk::ColorSpaceKHR {
//         self.color_space
//     }

//     #[inline]
//     pub fn extent(&self) -> vk::Extent2D {
//         self.extent
//     }

//     #[inline]
//     pub fn present_mode(&self) -> vk::PresentModeKHR {
//         self.present_mode
//     }

//     #[inline]
//     pub fn device(&self) -> &Arc<Device> {
//         &self.device
//     }

//     #[inline]
//     pub fn surface(&self) -> &Arc<Surface> {
//         &self.surface
//     }

//     /// Acquire the next image from the swapchain.
//     ///
//     /// Returns the index of the acquired image.
//     pub fn acquire_next_image(
//         &self,
//         timeout: u64,
//         semaphore: Option<vk::Semaphore>,
//         fence: Option<vk::Fence>,
//     ) -> Result<(u32, bool), SwapchainError> {
//         let khr_swapchain_device = self
//             .device
//             .khr_swapchain_device()
//             .ok_or(SwapchainError::extension_not_enabled(khr::swapchain::NAME))?;

//         Ok(unsafe {
//             khr_swapchain_device.acquire_next_image(
//                 self.swapchain,
//                 timeout,
//                 semaphore.unwrap_or(vk::Semaphore::null()),
//                 fence.unwrap_or(vk::Fence::null()),
//             )?
//         })
//     }

//     /// Present the swapchain image to the surface.
//     ///
//     /// Returns true if the swapchain is suboptimal.
//     pub fn queue_present(
//         &self,
//         queue: vk::Queue,
//         image_index: u32,
//         wait_semaphores: &[vk::Semaphore],
//     ) -> Result<bool, SwapchainError> {
//         let khr_swapchain_device = self
//             .device
//             .khr_swapchain_device()
//             .ok_or(SwapchainError::extension_not_enabled(khr::swapchain::NAME))?;

//         let swapchains = [self.swapchain];
//         let image_indices = [image_index];

//         let present_info = vk::PresentInfoKHR::default()
//             .wait_semaphores(wait_semaphores)
//             .swapchains(&swapchains)
//             .image_indices(&image_indices);

//         Ok(unsafe { khr_swapchain_device.queue_present(queue, &present_info)? })
//     }
// }

// impl Drop for Swapchain {
//     fn drop(&mut self) {
//         unsafe {
//             self.device
//                 .khr_swapchain_device()
//                 .unwrap()
//                 .destroy_swapchain(self.swapchain, None);
//         }
//     }
// }

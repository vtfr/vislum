// use std::sync::{Arc, RwLock};

// use vulkano::{
//     device::Device, image::{Image, ImageUsage}, swapchain::{
//         CompositeAlpha, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
//     }, sync::{fence::Fence, semaphore::Semaphore}
// };
// use winit::window::Window;

// use crate::screen::{AcquireError, AcquiredImage, FrameIndex, PresentError, Screen};

// struct FrameInfo {
//     image: Arc<Image>,
//     acquire_fence: Fence,
//     acquire_semaphore: Arc<Semaphore>,
// }

// struct WindowScreenSwapchain {
//     swapchain: Arc<Swapchain>,
//     frames: Vec<FrameInfo>,
// }

// /// A screen that renders to a window.
// pub struct WindowScreen {
//     device: Arc<Device>,
//     swapchain: RwLock<WindowScreenSwapchain>,
// }

// impl WindowScreen {
//     pub fn new(
//         device: Arc<Device>,
//         surface: Arc<Surface>,
//         window: Arc<Window>,
//     ) -> Result<Self, vulkano::Validated<vulkano::VulkanError>> {
//         let (swapchain, images) =
//             Self::create_swapchain(device.clone(), surface.clone(), &*window)?;

//         let frames = images.iter()
//             .map(|image| FrameInfo {
//             image: image.clone(),
//             acquire_fence: Arc::new(Fence::new(device.clone(), create_info)
//             acquire_semaphore: Arc::new(Semaphore::new(device.clone())?),
//         })
//         .collect();

//         Ok(Self {
//             device,
//             swapchain,
//         })
//     }

//     fn create_swapchain(
//         device: Arc<Device>,
//         surface: Arc<Surface>,
//         window: &Window,
//     ) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>), vulkano::Validated<vulkano::VulkanError>> {
//         // Query surface capabilities
//         let surface_capabilities = device
//             .physical_device()
//             .surface_capabilities(&surface, Default::default())?;

//         // Get window size
//         let window_size = window.inner_size();
//         let image_extent = surface_capabilities
//             .current_extent
//             .unwrap_or([window_size.width, window_size.height]);

//         // Choose minimum image count (prefer double-buffering)
//         let min_image_count = match surface_capabilities.max_image_count {
//             None => std::cmp::max(2, surface_capabilities.min_image_count),
//             Some(limit) => std::cmp::min(
//                 std::cmp::max(2, surface_capabilities.min_image_count),
//                 limit,
//             ),
//         };

//         // Query surface formats
//         let surface_formats = device
//             .physical_device()
//             .surface_formats(&surface, Default::default())?;

//         // Prefer sRGB format, fallback to first available
//         // Check if format name contains "SRGB" as a simple heuristic
//         let (image_format, image_color_space) = surface_formats
//             .iter()
//             .find(|(format, _)| {
//                 let format_str = format!("{:?}", format);
//                 format_str.contains("SRGB") || format_str.contains("SRGB")
//             })
//             .copied()
//             .unwrap_or(surface_formats[0]);

//         // Choose present mode (prefer Mailbox for low latency, fallback to FIFO)
//         let present_modes = device
//             .physical_device()
//             .surface_present_modes(&surface, Default::default())?;

//         let present_mode = present_modes
//             .iter()
//             .find(|&&mode| mode == PresentMode::Mailbox)
//             .copied()
//             .unwrap_or(PresentMode::Fifo);

//         // Use identity transform (no rotation)
//         let pre_transform = surface_capabilities.current_transform;

//         // Choose composite alpha mode
//         let composite_alpha = surface_capabilities
//             .supported_composite_alpha
//             .into_iter()
//             .find(|&alpha| alpha == CompositeAlpha::Opaque)
//             .unwrap_or_else(|| {
//                 surface_capabilities
//                     .supported_composite_alpha
//                     .into_iter()
//                     .next()
//                     .unwrap()
//             });

//         let create_info = SwapchainCreateInfo {
//             min_image_count,
//             image_format,
//             image_color_space,
//             image_extent,
//             image_array_layers: 1,
//             image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
//             image_sharing: vulkano::sync::Sharing::Exclusive,
//             pre_transform,
//             composite_alpha,
//             present_mode,
//             clipped: true,
//             ..Default::default()
//         };

//         Swapchain::new(device, surface, create_info)
//     }

//     /// Recreates the swapchain, typically after a window resize.
//     pub fn recreate_swapchain(&mut self, window: &Window) -> Result<(), vulkano::Validated<vulkano::VulkanError>> {
//         let window_size = window.inner_size();

//         let new_create_info = SwapchainCreateInfo {
//             image_extent: [window_size.width, window_size.height],
//             ..self.swapchain.create_info()
//         };

//         match self.swapchain.recreate(new_create_info) {
//             Ok((new_swapchain, new_images)) => {
//                 self.swapchain = new_swapchain;
//                 self.images = new_images;
//                 Ok(())
//             }
//             Err(e) => Err(e),
//         }
//     }
// }

// impl Screen for WindowScreen {
//     fn acquire_image(&self) -> Result<AcquiredImage, AcquireError> {
//         // Get a new fence and semaphore from the pool for this acquisition
//         let acquire_fence = Arc::new(Fence::from_pool(self.device.clone())
//             .map_err(|e| AcquireError::ValidationError(vulkano::Validated::VulkanError(e)))?);
//         let acquire_semaphore = Arc::new(Semaphore::from_pool(self.device.clone())
//             .map_err(|e| AcquireError::ValidationError(vulkano::Validated::VulkanError(e)))?);

//         let acquire_info = vulkano::swapchain::AcquireNextImageInfo {
//             timeout: None,
//             semaphore: Some(acquire_semaphore.clone()),
//             fence: Some(acquire_fence.clone()),
//             ..Default::default()
//         };

//         let acquired_image = unsafe { self.swapchain.acquire_next_image(&acquire_info)? };
//         let index = FrameIndex(acquired_image.image_index as u8);
//         let frame = &self.frames[index.0 as usize];

//         // Store the fence and semaphore for this frame (for use in present)
//         *frame.acquire_fence.lock().unwrap() = acquire_fence.clone();
//         *frame.acquire_semaphore.lock().unwrap() = acquire_semaphore.clone();

//         // Wait for the fence to be signaled, indicating the image is ready
//         acquire_fence.wait(None)
//             .map_err(|e| AcquireError::ValidationError(vulkano::Validated::VulkanError(e)))?;

//         Ok(AcquiredImage {
//             image: frame.image.clone(),
//             index,
//         })
//     }

//     fn present(
//         &self,
//         queue: Arc<vulkano::device::Queue>,
//         index: FrameIndex,
//     ) -> Result<(), PresentError> {
//         let frame = &self.frames[index.0 as usize];
        
//         // Get the acquire semaphore for this frame (signaled when image was acquired)
//         // This semaphore should be waited on before presenting to ensure the image is ready
//         let acquire_semaphore = frame.acquire_semaphore.lock().unwrap().clone();
        
//         let present_info = vulkano::swapchain::PresentInfo {
//             swapchain_infos: vec![
//                 vulkano::swapchain::SwapchainPresentInfo::swapchain_image_index(
//                     self.swapchain.clone(),
//                     index.into(),
//                 ),
//             ],
//             wait_semaphores: vec![
//                 vulkano::swapchain::SemaphorePresentInfo::new(acquire_semaphore),
//             ],
//             ..Default::default()
//         };

//         queue.with(|mut queue| unsafe { queue.present(&present_info) })?;

//         Ok(())
//     }
// }

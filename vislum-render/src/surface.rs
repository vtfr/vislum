use std::sync::Arc;

use vulkano::{
    device::Device,
    image::{Image, ImageUsage, view::ImageView},
    swapchain::{Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo},
};

pub trait Screen {
    fn acquire_next_frame(&self) -> Option<ScreenFrameInfo>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameIndex(pub u32);

pub struct ScreenFrameInfo {
    /// The index of the frame in the swapchain.
    index: FrameIndex,
    /// The image of the frame.
    image: Arc<Image>,
    /// The image view of the frame.
    image_view: Arc<ImageView>,
    /// The swapchain of the frame.
    swapchain: Arc<Swapchain>,
}

pub struct SurfaceSwapchainScreen {
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    window: Arc<winit::window::Window>,
    frames: Vec<ScreenFrameInfo>,
}

impl SurfaceSwapchainScreen {
    /// Creates a new `SurfaceSwapchainScreen` from a `Device` and a `Window`.
    pub fn new(
        device: Arc<Device>,
        window: Arc<winit::window::Window>,
        surface: Arc<Surface>,
    ) -> Self {
        let physical_device = device.physical_device();

        let surface_capabilities = physical_device
            .surface_capabilities(&*surface, SurfaceInfo::default())
            .unwrap();

        let surface_formats = physical_device
            .surface_formats(&*surface, SurfaceInfo::default())
            .unwrap();

        let min_image_count = u32::clamp(
            3,
            surface_capabilities.min_image_count + 1,
            surface_capabilities
                .max_image_count
                .unwrap_or(std::u32::MAX),
        );

        let find_present_mode =
            |mode: vulkano::swapchain::PresentMode| -> Option<vulkano::swapchain::PresentMode> {
                surface_capabilities
                    .compatible_present_modes
                    .iter()
                    .copied()
                    .find(|m| *m == mode)
            };

        let present_mode = find_present_mode(vulkano::swapchain::PresentMode::Mailbox)
            .or_else(|| find_present_mode(vulkano::swapchain::PresentMode::Fifo))
            .unwrap_or(vulkano::swapchain::PresentMode::Immediate);

        let create_info = SwapchainCreateInfo {
            min_image_count,
            image_format: surface_formats[0].0,
            image_view_formats: vec![surface_formats[0].0],
            image_color_space: surface_formats[0].1,
            image_extent: [window.inner_size().width, window.inner_size().height],
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            present_mode,
            present_modes: [present_mode].into_iter().collect(),
            scaling_behavior: None,
            ..Default::default()
        };

        let (swapchain, images) = Swapchain::new(device, surface.clone(), create_info).unwrap();

        let frames = images
            .into_iter()
            .enumerate()
            .map(|(index, image)| ScreenFrameInfo {
                index: FrameIndex(index as u32),
                image_view: ImageView::new(image.clone(), Default::default()).unwrap(),
                image,
                swapchain: swapchain.clone(),
            })
            .collect();

        Self {
            window,
            surface,
            swapchain,
            frames,
        }
    }
}

impl Screen for SurfaceSwapchainScreen {
    fn acquire_next_frame(&self) -> Option<ScreenFrameInfo> {
        let frame = self.swapchain.acquire_next_image(self.image_available[frame].vk_handle()).unwrap();
        Some(self.frames[frame.index])
    }
}
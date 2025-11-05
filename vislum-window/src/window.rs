use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use vislum_render_rhi::{
    device::Device,
    image::{Image, ImageView, ImageViewCreateInfo, ImageViewType},
    instance::Instance,
    queue::Queue,
    surface::Surface,
    swapchain::{Swapchain, SwapchainCreateInfo},
    sync::{Fence, Semaphore},
};

use crate::app::Application;

/// Information for a single swapchain frame.
pub struct SwapchainFrameInfo {
    pub image: Arc<Image>,
    pub image_view: Arc<ImageView>,
    pub acquire_semaphore: Arc<Semaphore>,
    pub render_semaphore: Arc<Semaphore>,
}

/// Render context passed to Application::render() containing current frame info.
pub struct RunnerRenderContext {
    pub image: Arc<Image>,
    pub image_view: Arc<ImageView>,
    pub acquire_semaphore: Arc<Semaphore>,
    pub render_semaphore: Arc<Semaphore>,
}

/// Context containing all initialized rendering objects.
pub struct RunnerContext {
    pub window: Arc<Window>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface>,
    pub swapchain: Arc<Swapchain>,
    pub frame_infos: Vec<SwapchainFrameInfo>,
}

/// Window runner containing all initialized objects for rendering.
pub struct Runner<A: Application> {
    pub context: Option<RunnerContext>,
    pub app: Option<A>,
    frame_index: usize,
    // Initialization parameters
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    window_attributes: winit::window::WindowAttributes,
}

impl<A: Application> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_none() {
            log::info!("Creating window...");
            let window = Arc::new(event_loop.create_window(self.window_attributes.clone()).unwrap());
            log::info!("Window created");

            log::info!("Creating surface...");
            let surface = Surface::new(self.instance.clone(), &window);
            log::info!("Surface created");

            log::info!("Creating swapchain...");
            let (swapchain, swapchain_images) = Swapchain::new(
                self.device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: None,
                    present_mode: None,
                    image_usage: None,
                    old_swapchain: None,
                },
            );
            log::info!("Swapchain created with {} images", swapchain_images.len());

            // Create per-frame info (one set per swapchain image)
            log::info!("Creating per-frame info...");
            let frame_infos: Vec<SwapchainFrameInfo> = swapchain_images
                .iter()
                .map(|image| {
                    // Create image view for this swapchain image
                    let image_view = ImageView::new(
                        self.device.clone(),
                        ImageViewCreateInfo {
                            image: image.clone(),
                            view_type: ImageViewType::D2,
                            format: swapchain.image_format(),
                            components: vk::ComponentMapping::default(),
                            subresource_range: vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_mip_level(0)
                                .level_count(1)
                                .base_array_layer(0)
                                .layer_count(1),
                        },
                    );

                    SwapchainFrameInfo {
                        image: image.clone(),
                        image_view,
                        acquire_semaphore: Semaphore::new(self.device.clone()),
                        render_semaphore: Semaphore::new(self.device.clone()),
                    }
                })
                .collect();
            log::info!("Created {} frame info sets", frame_infos.len());

            // Create the application
            log::info!("Creating application...");
            match A::new(self.device.clone(), self.queue.clone()) {
                Ok(app) => {
                    self.app = Some(app);
                    log::info!("Application created successfully");
                }
                Err(e) => {
                    log::error!("Failed to create application: {:?}", e);
                    event_loop.exit();
                    return;
                }
            }

            self.context = Some(RunnerContext {
                window,
                device: self.device.clone(),
                queue: self.queue.clone(),
                surface,
                swapchain,
                frame_infos,
            });
        }

        // Request redraw when about to wait (before first frame)
        if let Some(ref context) = self.context {
            context.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut context) = self.context else { return; };
        let Some(ref mut app) = self.app else { return; };

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                log::debug!("RedrawRequested event received");

                // Get sync objects for current frame slot
                let frame_info = &context.frame_infos[self.frame_index];

                // Acquire next swapchain image
                log::debug!("Acquiring swapchain image...");
                let (img_idx, suboptimal) = context.swapchain.acquire_next_image(
                    u64::MAX,
                    Some(&frame_info.acquire_semaphore),
                    None,
                );
                log::debug!("Acquired swapchain image {}", img_idx);

                if suboptimal {
                    log::warn!("Swapchain is suboptimal");
                    // Could recreate swapchain here if needed
                }

                // Get frame info for the acquired swapchain image
                let swapchain_frame_info = &context.frame_infos[img_idx as usize];

                // Create render context with current frame info
                let render_ctx = RunnerRenderContext {
                    image: swapchain_frame_info.image.clone(),
                    image_view: swapchain_frame_info.image_view.clone(),
                    acquire_semaphore: frame_info.acquire_semaphore.clone(),
                    render_semaphore: frame_info.render_semaphore.clone(),
                };

                // Call user's render function
                // The application is responsible for managing its own render context and frame graph
                if let Err(e) = app.render(&render_ctx) {
                    log::error!("Render error: {:?}", e);
                }

                // Present
                log::debug!("Presenting swapchain image...");
                context
                    .swapchain
                    .present(&context.queue, img_idx, &[&render_ctx.render_semaphore]);

                // Advance to next frame slot
                self.frame_index = (self.frame_index + 1) % context.frame_infos.len();

                // Request redraw
                context.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl<A: Application> Runner<A> {
    /// Runs the application with the event loop.
    /// 
    /// Handles:
    /// - Window creation in `resumed()` callback
    /// - Application creation in `resumed()` callback
    /// - Window events (close, resize)
    /// - RedrawRequested events → calls `app.render()`
    /// - Swapchain image acquisition, synchronization, presentation
    /// 
    /// The application is responsible for managing its own render context and frame graph.
    pub fn run(
        event_loop: EventLoop<()>,
        instance: Arc<Instance>,
        device: Arc<Device>,
        queue: Arc<Queue>,
        window_attributes: winit::window::WindowAttributes,
    ) -> Result<()> {
        let mut runner = Runner {
            context: None,
            app: None,
            frame_index: 0,
            instance,
            device,
            queue,
            window_attributes,
        };

        event_loop.run_app(&mut runner)?;

        Ok(())
    }
}

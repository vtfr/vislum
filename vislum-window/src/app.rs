use std::sync::Arc;

use crate::window::RunnerRenderContext;
use anyhow::Result;
use vislum_render_rhi::{device::Device, queue::Queue};

/// Trait for applications that render to a window.
/// 
/// Users implement `render()` which is called each frame during the render loop.
pub trait Application {
    /// Creates a new application instance with the render dependencies.
    /// 
    /// This is called once during window initialization. The application can use
    /// this to set up its resources, including creating its own render context.
    fn new(device: Arc<Device>, queue: Arc<Queue>) -> Result<Self>
    where
        Self: Sized;

    /// Called each frame to render the scene.
    /// 
    /// The application is responsible for managing its own render context and
    /// frame graph. The `render_ctx` provides the current frame's swapchain
    /// image, image view, and sync objects (all as Arcs).
    fn render(&mut self, render_ctx: &RunnerRenderContext) -> Result<()>;
}


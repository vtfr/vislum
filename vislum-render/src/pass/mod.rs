pub mod blit;
pub mod draw;
pub mod forward;

pub use blit::*;
pub use draw::*;
pub use forward::*;

use std::sync::Arc;
use vislum_system::{Resource, Resources};

use crate::{RenderDevice, RenderQueue};

/// The final output for the render graph.
///
/// Generally the screen.
pub struct ScreenRenderTarget {
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

pub trait RenderPass {
    fn render(
        &self,
        resources: &Resources,
        screen_render_target: &ScreenRenderTarget,
        encoder: &mut wgpu::CommandEncoder,
    );
}

/// Collects render passes and renders them in order.
#[derive(Resource, Default)]
pub struct RenderPassCollector {
    passes: Vec<Arc<dyn RenderPass>>,
}

impl RenderPassCollector {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_pass(&mut self, pass: Arc<dyn RenderPass>) {
        self.passes.push(pass);
    }

    /// Renders the render passes in order.
    ///
    /// Drains the passes after rendering, so that the collector can be reused.
    pub fn render(
        &mut self,
        resources: &Resources,
        screen_render_target: &ScreenRenderTarget,
    ) {
        // Create a new encoder.
        let mut encoder = resources.get::<RenderDevice>()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Render the passes.
        for pass in self.passes.drain(..) {
            pass.render(resources, &screen_render_target, &mut encoder);
        }

        // Submit the encoder.
        resources.get::<RenderQueue>()
            .submit(std::iter::once(encoder.finish()));
    }
}

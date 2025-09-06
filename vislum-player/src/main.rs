use std::{error::Error, sync::Arc};

use vislum_op::{
    compile::CompilationContext,
    eval::{Eval, EvalContext, EvalError, Multiple, Output, Single},
    prelude::*,
    system::NodeGraphSystem,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event,
    event_loop::{EventLoop, EventLoopProxy},
    window::{Fullscreen, Window, WindowAttributes},
};

use crate::wgpu_state::WgpuState;

mod wgpu_state;

#[derive(Default)]
pub enum PlayerState {
    #[default]
    Uninitialized,
    /// The window has been created, but the runtime has not been initialized
    WindowCreated { window: Arc<Window> },
    /// The runtime has been initialized
    WgpuInitialized {
        wgpu: WgpuState,
        window: Arc<Window>,
    },
}

impl PlayerState {
    pub fn window(&self) -> Option<&Window> {
        match self {
            PlayerState::WindowCreated { window } => Some(window),
            PlayerState::WgpuInitialized { window, .. } => Some(window),
            PlayerState::Uninitialized => None,
        }
    }

    pub fn wgpu(&self) -> Option<&WgpuState> {
        match self {
            PlayerState::WgpuInitialized { wgpu, .. } => Some(wgpu),
            _ => None,
        }
    }
}

pub struct Player {
    proxy: EventLoopProxy<PlayerEvent>,
    state: PlayerState,
}

impl Player {
    fn new(proxy: EventLoopProxy<PlayerEvent>) -> Self {
        Self {
            proxy,
            state: PlayerState::Uninitialized,
        }
    }
}

impl Player {
    fn render(&self) {
        // Request a redraw of the window
        if let Some(window) = self.state.window() {
            window.request_redraw();
        }

        if let PlayerState::WgpuInitialized { wgpu, .. } = &self.state {
            let current_texture = wgpu.surface.get_current_texture().unwrap();
            let view = current_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    label: None,
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }

            wgpu.queue.submit(std::iter::once(encoder.finish()));
            current_texture.present();
        }
    }
}

impl ApplicationHandler<PlayerEvent> for Player {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let PlayerState::Uninitialized = self.state {
            let window_attributes = WindowAttributes::default()
                .with_active(true)
                .with_visible(true)
                .with_title("Vislum Player")
                .with_resizable(false)
                .with_inner_size(PhysicalSize::new(800, 600))
                .with_fullscreen(None);

            // Create the window
            let window = event_loop.create_window(window_attributes).unwrap();
            let window = Arc::new(window);

            self.state = PlayerState::WindowCreated {
                window: window.clone(),
            };

            // Initialize the runtime.
            let wgpu = pollster::block_on(WgpuState::new(window.clone())).unwrap();

            self.state = PlayerState::WgpuInitialized {
                window: window.clone(),
                wgpu,
            };
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: PlayerEvent) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: event::WindowEvent,
    ) {
        match &event {
            event::WindowEvent::Resized(_physical_size) => {
                // TODO
            }
            event::WindowEvent::RedrawRequested => {
                self.render();
            }
            event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

pub enum PlayerEvent {
    WindowCreated,
}

fn run() -> anyhow::Result<()> {
    let event_loop: EventLoop<PlayerEvent> = EventLoop::with_user_event().build()?;

    let proxy = event_loop.create_proxy();

    event_loop.run_app(&mut Player::new(proxy))?;

    Ok(())
}

fn main() {
    run().unwrap();
}

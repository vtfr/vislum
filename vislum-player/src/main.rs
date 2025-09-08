use std::{error::Error, sync::Arc};

use vislum_op::{
    compile::CompilationContext,
    eval::{Eval, EvalContext, EvalError, Multiple, Output, Single},
    prelude::*,
    system::NodeGraphSystem,
};
use vislum_render::{Handle, MeshManager, RenderPassCollector, SceneManager, ScreenRenderTarget, TextureManager};
use vislum_render::texture::{Texture, TextureDescriptor, TextureFormat};
use vislum_render::scene::{Scene, SceneCommand, SceneObject};
use vislum_render::mesh::{MeshDescriptor, Vertex};
use vislum_render::pass::{ForwardRenderPass, ScreenBlitPass};
use vislum_runtime::Runtime;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event,
    event_loop::{EventLoop, EventLoopProxy},
    window::{Fullscreen, Window, WindowAttributes},
};

use crate::wgpu_state::WgpuState;

mod wgpu_state;

/// Testing data for the player.
struct TestingData {
    render_texture: Handle<Texture>,
    scene: Handle<Scene>,
}

#[derive(Default)]
enum PlayerState {
    #[default]
    Uninitialized,
    /// The window has been created, but the runtime has not been initialized
    Ready {
        window: Arc<Window>,
        wgpu: WgpuState,
        runtime: Runtime,
        testing_data: TestingData,
    },
}

struct Player {
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
        let PlayerState::Ready { window, wgpu, runtime, testing_data } = &self.state else { return };
            // Request a redraw of the window for the next frame
            window.request_redraw();
        
            // Get the current texture
            let current_texture = wgpu.surface.get_current_texture().unwrap();
            let view = current_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass_collector = runtime.get_resource_mut::<RenderPassCollector>();
            
            // Do render the scene.
            // render_pass_collector.add_pass(Arc::new(ForwardRenderPass { 
            //     scene: testing_data.scene.clone(), 
            //     color_texture: testing_data.render_texture.clone(),
            // }));

            render_pass_collector.add_pass(Arc::new(ForwardRenderPass { 
                scene: testing_data.scene.clone(), 
                color: testing_data.render_texture.clone(),
            }));
            render_pass_collector.add_pass(Arc::new(ScreenBlitPass::new(testing_data.render_texture.clone())));
            render_pass_collector.render(&runtime.resources, &ScreenRenderTarget {
                view,
                format: current_texture.texture.format(),
            });

            // Present the texture.
            current_texture.present();
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

            // Initialize the runtime.
            let wgpu = pollster::block_on(WgpuState::new(window.clone())).unwrap();
            let runtime = Runtime::new(wgpu.device.clone(), wgpu.queue.clone());

            // TESTING DATA
            let mut texture_manager = runtime.get_resource_mut::<TextureManager>();
            let mut mesh_manager = runtime.get_resource_mut::<MeshManager>();
            let mut scene_manager = runtime.get_resource_mut::<SceneManager>();

            let color_texture = texture_manager.create(TextureDescriptor {
                format: TextureFormat::Rgba8Unorm,
                data: None,
                width: 1920,
                height: 1080,
            });

            let mesh = mesh_manager.create(MeshDescriptor {
                vertices: vec![
                    Vertex { position: [-0.5, -0.5, 0.0] },
                    Vertex { position: [0.5, -0.5, 0.0] }, 
                    Vertex { position: [0.0, 0.5, 0.0] }
                ],
                indices: vec![0, 1, 2],
            });

            let scene = scene_manager.create_with_commands(vec![
                SceneCommand::AddObject(SceneObject {
                    mesh,
                }),
            ]);

            drop(texture_manager);
            drop(mesh_manager);
            drop(scene_manager);

            self.state = PlayerState::Ready {
                window: window.clone(),
                wgpu,
                runtime,
                testing_data: TestingData {
                    render_texture: color_texture,
                    scene,
                },
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

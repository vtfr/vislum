// pub struct App {}

// impl App {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl ApplicationHandler for App {
//     fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
//     }

//     fn window_event(
//         &mut self,
//         event_loop: &winit::event_loop::ActiveEventLoop,
//         window_id: winit::window::WindowId,
//         event: winit::event::WindowEvent,
//     ) {
//         todo!()
//     }
// }

use std::sync::Arc;

use vislum_render::rhi::{Instance, InstanceDescription, InstanceFeatures};
use winit::{
    application::ApplicationHandler,
    platform::wayland::WindowAttributesExtWayland,
    window::{Window, WindowAttributes},
};

struct TestApp {
    instance: Arc<Instance>,
    window: Option<Window>,
    
}

impl TestApp {
    fn new(instance: Arc<Instance>) -> Self {
        Self {
            instance,
            window: Default::default(),
        }
    }
}

impl ApplicationHandler for TestApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut attributes = WindowAttributes::default().with_title("Vislum test renderer");

        let window = event_loop.create_window(attributes).unwrap();

        let physical_device = self
            .instance
            .enumerate_physical_devices()
            .into_iter()
            .next()
            .unwrap();
        let device = self.instance.create_device(physical_device).unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        todo!()
    }
}

fn main() {
    let instance = Instance::new(InstanceDescription {
        application_name: "Vislum test renderer".into(),
        features: InstanceFeatures { surface: true },
    })
    .unwrap();

    let mut app = TestApp::new(instance);

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.run_app(&mut app);
}

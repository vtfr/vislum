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

use log::LevelFilter;
use vislum_render::rhi::{Instance, InstanceDescription};
use winit::{
    application::ApplicationHandler,
    platform::wayland::{EventLoopExtWayland},
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
        // let device = self.instance.create_device(physical_device).unwrap();
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
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(LevelFilter::Debug)
        .init();

    // let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let instance = Instance::new(InstanceDescription {
        application_name: "Vislum test renderer".into(),
    })
    .unwrap();

    let physical_devices = instance.enumerate_physical_devices();
    for (index, physical_device) in physical_devices.iter().enumerate() {
        println!("Physical device {index}: {physical_device:#?}");
    }

    let device = instance.create_device(physical_devices.into_iter().next().unwrap());

    // let mut app = TestApp::new(instance);

    // event_loop.run_app(&mut app);
}

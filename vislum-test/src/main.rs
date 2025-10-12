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

use log::LevelFilter;
use vislum_render::rhi::{device::Device, instance::{Instance, InstanceExtensions}};

fn main() {
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(LevelFilter::Debug)
        .filter_module("vislum_render", LevelFilter::Debug)
        .parse_default_env()
        .init();

    // let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let extensions = InstanceExtensions {
        khr_swapchain: true,
        khr_surface: true,
        khr_wayland_surface: true,
        ..Default::default()
    };

    let instance = unsafe { Instance::new() }.unwrap();

    let physical_devices = instance.enumerate_compatible_devices().unwrap();
    for (i, physical_device) in physical_devices.iter().enumerate() {
        println!("Physical device {i}: {:#?}", physical_device);
    }

    let device = Device::from_physical_device(instance, physical_devices[0].clone());
    println!("Device: {:#?}", device);
}

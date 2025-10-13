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
use vislum_render::rhi::instance::{Instance, InstanceDescription, InstanceExtensions};

fn main() {
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(LevelFilter::Debug)
        .filter_module("vislum_render", LevelFilter::Debug)
        .parse_default_env()
        .init();

    // let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let instance = Instance::new(InstanceDescription {
        extensions: InstanceExtensions {
            khr_surface: true,
            khr_wayland_surface: true,
            ext_debug_utils: true,
            khr_get_physical_device_properties2: true,
            khr_get_surface_capabilities2: true,
            ext_swapchain_colorspace: true,
            khr_portability_enumeration: true,
            ..Default::default()
        },
    })
    .unwrap();

    let physical_devices = instance.enumerate_physical_devices();
    for (i, physical_device) in physical_devices.iter().enumerate() {
        log::info!("Physical device {i}:");
        log::info!(
            "  Supported extensions: {}",
            physical_device.supported_extensions()
        );
        log::info!(
            "  Supported features: {}",
            physical_device.supported_features()
        );
        log::info!("  Version: {}", physical_device.version());
    }

    let device = instance.create_device(physical_devices[0].clone());
    log::info!("Device: {:?}", device);
}

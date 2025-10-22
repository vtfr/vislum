use vislum_render::rhi::{device::{DeviceDescription, DeviceExtensions, DeviceFeatures}, instance::Instance};

fn main() {
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .filter_module("vislum_render", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    // let event_loop = EventLoop::new().expect("Failed to create event loop");
    // let mut app = TriangleDemo::new();

    let instance = Instance::new().expect("Failed to create instance");
    log::info!("Instance created: {:#?}", instance);

    let physical_devices = instance.enumerate_physical_devices();
    for (index, physical_device) in physical_devices.iter().enumerate() {
        log::info!("Physical device {}:", index);
        log::info!("  Name: {}", physical_device.name());
        log::info!("  API version: {}", physical_device.version());
        log::info!("  Extensions: {}", physical_device.supported_extensions());
        log::info!("  Features: {}", physical_device.supported_features());
    }

    let device = instance.create_device(DeviceDescription {
        physical_device: physical_devices.into_iter().next().unwrap(),
        features: DeviceFeatures::default(),
        extensions: DeviceExtensions::default(),
    }).expect("Failed to create device");
    // log::info!("Starting triangle demo");
    // event_loop
    //     .run_app(&mut app)
    //     .expect("Failed to run event loop");
}

use vislum_render_rhi::{Version, device::{Device, DeviceCreateInfo, DeviceExtensions, FeatureStorage}, instance::{Instance, InstanceExtensions, Library}};

fn main() {
    let library = Library::new();
    let instance = Instance::new(library, InstanceExtensions::default());

    for physical_device in instance.enumerate_physical_devices() {
        println!("Physical device:");
        println!("  Properties:");
        println!("    API version: {}", physical_device.properties().api_version);
        println!("    Driver version: {}", physical_device.properties().driver_version);
        println!("    Vendor ID: {}", physical_device.properties().vendor_id);
        println!("    Device ID: {}", physical_device.properties().device_id);
        println!("    Device type: {:?}", physical_device.properties().device_type);
        println!("    Device name: {}", physical_device.properties().device_name);
        println!("  Extensions:");
        println!("    {}", physical_device.extensions());
        println!("  Capabilities:");
        for capability in physical_device.capabilities() {
            println!("    Queue flags: {:?}", capability.queue_flags);
            println!("    Queue count: {}", capability.queue_count);
        }
        println!("  Features: {:#?}", physical_device.supported_features());
    }

    let physical_device = instance.enumerate_physical_devices().next().unwrap();

    let features = physical_device.supported_features();

    let device = Device::new(instance, DeviceCreateInfo {
        api_version: Version::V1_3,
        physical_device,
        extensions: DeviceExtensions::default(),
        features,
    });

}
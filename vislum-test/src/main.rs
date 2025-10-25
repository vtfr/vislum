use vislum_rhi::instance::{Instance, InstanceExtensions, Library};

fn main() {
    let library = Library::new();
    let instance = Instance::new(library, InstanceExtensions {
        khr_surface: true,
        ..Default::default()
    });
    let physical_devices = instance.enumerate_physical_devices();
    println!("Physical devices: {:#?}", physical_devices);
}

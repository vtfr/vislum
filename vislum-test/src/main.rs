use std::{ffi::CStr, mem::MaybeUninit};

use vislum_dxc::sys::{DxcShimCompiler, DxcShimLoader};

fn main() {
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .filter_module("vislum_render", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let mut loader = MaybeUninit::<*mut DxcShimLoader>::uninit();
    unsafe { vislum_dxc::sys::dxc_shim_loader_open(loader.as_mut_ptr()) };

    let loader = unsafe { loader.assume_init() };

    let mut compiler = MaybeUninit::<*mut DxcShimCompiler>::uninit();
    unsafe { vislum_dxc::sys::dxc_shim_create_compiler(loader, compiler.as_mut_ptr()) };

    let compiler = unsafe { compiler.assume_init() };

    let source = c"
    #include \"common.hlsl\"
    struct Vertex {
        float3 position : POSITION;
    };

    void main(in Vertex vertex : POSITION) {
        return;
    }";
    let result = unsafe { vislum_dxc::sys::dxc_shim_compile(compiler, source.as_ptr() as *const _) };

    if unsafe { vislum_dxc::sys::dxc_shim_compilation_result_is_successful(result) } {
        let mut bytecode = MaybeUninit::<*mut std::ffi::c_void>::uninit();
        let mut size = MaybeUninit::<usize>::uninit();
        unsafe { vislum_dxc::sys::dxc_shim_compilation_result_get_bytecode(result, bytecode.as_mut_ptr(), size.as_mut_ptr()) };
        let size = unsafe { size.assume_init() };
        log::info!("Size: {}", size);
        log::info!("Compilation successful");
    } else {
        let error_message_c = unsafe { vislum_dxc::sys::dxc_shim_compilation_result_get_error_message(result) };
        let error_message = unsafe { CStr::from_ptr(error_message_c) }.to_string_lossy();
        log::error!("Compilation failed: {}", error_message);
    }

    // let event_loop = EventLoop::new().expect("Failed to create event loop");
    // let mut app = TriangleDemo::new();

    // let instance = Instance::new().expect("Failed to create instance");
    // log::info!("Instance created: {:#?}", instance);

    // let physical_devices = instance.enumerate_physical_devices();
    // for (index, physical_device) in physical_devices.iter().enumerate() {
    //     log::info!("Physical device {}:", index);
    //     log::info!("  Name: {}", physical_device.name());
    //     log::info!("  API version: {}", physical_device.version());
    //     log::info!("  Extensions: {}", physical_device.supported_extensions());
    //     log::info!("  Features: {}", physical_device.supported_features());
    // }

    // let device = instance.create_device(DeviceDescription {
    //     physical_device: physical_devices.into_iter().next().unwrap(),
    //     features: DeviceFeatures::default(),
    //     extensions: DeviceExtensions::default(),
    // }).expect("Failed to create device");
    // log::info!("Starting triangle demo");
    // event_loop
    //     .run_app(&mut app)
    //     .expect("Failed to run event loop");
}

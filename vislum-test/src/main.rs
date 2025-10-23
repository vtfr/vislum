use std::{ffi::CStr, mem::MaybeUninit};

use vislum_dxc::sys::{DxcShimCompiler, DxcShimLoader};

fn main() {
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .filter_module("vislum_render", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let loader = vislum_dxc::DxcLoader::new().expect("Failed to create loader");

    let compiler = vislum_dxc::DxcCompiler::new(loader)
        .expect("Failed to create compiler");

    let source = r#"
#include "common.hlsl"
struct Vertex {
    float3 position : POSITION;
};

void main(in Vertex vertex : POSITION) {
    return;
}
"#;

    struct MyPrettyIncludeHandler;

    impl vislum_dxc::DxcIncludeHandler for MyPrettyIncludeHandler {
        fn load_source(&self, filename: &str) -> Option<String> {
            Some(format!("// Pretty include handler: {}\n", filename))
        }
    }

    let result = compiler.compile(source, &MyPrettyIncludeHandler).expect("Failed to compile");

    std::fs::write("output.spv", result).expect("Failed to write output.spv");

    log::info!("Compilation successful");
}

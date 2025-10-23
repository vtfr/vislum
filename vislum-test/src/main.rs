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

VertexOutput main(VertexInput input) {
    VertexOutput output;
    output.position = ApplyTransform(input.position);
    output.color = input.color;
    return output;
}
"#;

    struct MyPrettyIncludeHandler;

    impl vislum_dxc::DxcIncludeHandler for MyPrettyIncludeHandler {
        fn load_source(&self, filename: &str) -> Option<String> {
            match filename {
                "./common.hlsl" => Some(r#"
// Common HLSL utilities and structures

#ifndef COMMON_HLSL
#define COMMON_HLSL

// Common vertex input structure
struct VertexInput {
    float3 position : POSITION;
    float3 color : COLOR;
};

// Common vertex output structure
struct VertexOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR;
};

// Transformation matrices
struct Transform {
    float4x4 model;
    float4x4 view;
    float4x4 projection;
};

// Constant buffer for transformations
cbuffer TransformBuffer : register(b0) {
    Transform transform;
};

// Utility function to apply MVP transformation
float4 ApplyTransform(float3 position) {
    float4x4 mvp = mul(transform.projection, mul(transform.view, transform.model));
    return mul(mvp, float4(position, 1.0));
}

// Common color utilities
float3 HsvToRgb(float3 hsv) {
    float4 K = float4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    float3 p = abs(frac(hsv.xxx + K.xyz) * 6.0 - K.www);
    return hsv.z * lerp(K.xxx, clamp(p - K.xxx, 0.0, 1.0), hsv.y);
}

#endif // COMMON_HLSL
"#.to_string()),
                _ => None,
            }
        }
    }

    let result = compiler.compile(source, &MyPrettyIncludeHandler).expect("Failed to compile");

    std::fs::write("output.spv", result).expect("Failed to write output.spv");

    log::info!("Compilation successful");
}

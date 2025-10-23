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

// Bindless resource handling for Vulkan HLSL
// This include provides simple bindless texture and buffer access

#ifndef BINDLESS_HLSL
#define BINDLESS_HLSL

// Bindless texture descriptor set (set 1)
[[vk::binding(0, 1)]]
Texture2D<float4> bindless_textures[];

[[vk::binding(1, 1)]]
SamplerState bindless_samplers[];

// Bindless buffer descriptor set (set 2)
[[vk::binding(0, 2)]]
StructuredBuffer<float> bindless_structured_buffers[];

[[vk::binding(1, 2)]]
ByteAddressBuffer bindless_byte_buffers[];

// Helper functions for bindless resource access
float4 SampleBindlessTexture(uint textureIndex, uint samplerIndex, float2 uv)
{
    return bindless_textures[NonUniformResourceIndex(textureIndex)].Sample(
        bindless_samplers[NonUniformResourceIndex(samplerIndex)], uv);
}

float4 LoadBindlessTexture(uint textureIndex, int2 coord)
{
    return bindless_textures[NonUniformResourceIndex(textureIndex)].Load(int3(coord, 0));
}

float4 ReadBindlessStructuredBuffer(uint bufferIndex, uint elementIndex)
{
    return bindless_structured_buffers[NonUniformResourceIndex(bufferIndex)][elementIndex];
}

float4 ReadBindlessByteBuffer(uint bufferIndex, uint byteOffset)
{
    return bindless_byte_buffers[NonUniformResourceIndex(bufferIndex)].Load<float4>(byteOffset);
}

// Alternative naming for consistency
#define GetBindlessTexture SampleBindlessTexture
#define LoadBindlessTex LoadBindlessTexture
#define ReadBindlessBuffer ReadBindlessStructuredBuffer

#endif // BINDLESS_HLSL
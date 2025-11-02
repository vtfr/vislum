[[vk::binding(0, 0)]]
Texture2D textureSampler;

[[vk::binding(1, 0)]]
SamplerState textureSamplerState;

struct FragmentInput {
    float4 position : SV_POSITION;
    float2 uv : TEXCOORD0;
};

float4 main(FragmentInput input) : SV_Target {
    return textureSampler.Sample(textureSamplerState, input.uv);
}


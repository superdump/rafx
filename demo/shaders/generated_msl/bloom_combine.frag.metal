#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    texture2d<float> in_color [[id(0)]];
    sampler smp [[id(1)]];
    texture2d<float> in_blur [[id(2)]];
};

struct main0_out
{
    float4 out_sdr [[color(0)]];
};

struct main0_in
{
    float2 inUV [[user(locn0)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    main0_out out = {};
    float4 color = spvDescriptorSet0.in_color.sample(spvDescriptorSet0.smp, in.inUV) + spvDescriptorSet0.in_blur.sample(spvDescriptorSet0.smp, in.inUV);
    float3 mapped = color.xyz / (color.xyz + float3(1.0));
    out.out_sdr = float4(mapped, color.w);
    return out;
}


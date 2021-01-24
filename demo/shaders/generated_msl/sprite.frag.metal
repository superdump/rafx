#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    sampler smp [[id(0)]];
};

struct spvDescriptorSetBuffer1
{
    texture2d<float> tex [[id(0)]];
};

struct main0_out
{
    float4 uFragColor [[color(0)]];
};

struct main0_in
{
    float2 o_uv [[user(locn0)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]], constant spvDescriptorSetBuffer1& spvDescriptorSet1 [[buffer(1)]])
{
    main0_out out = {};
    float4 color = spvDescriptorSet1.tex.sample(spvDescriptorSet0.smp, in.o_uv);
    out.uFragColor = color;
    return out;
}


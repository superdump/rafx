#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct Args
{
    float4x4 mvp;
};

struct spvDescriptorSetBuffer0
{
    constant Args* uniform_buffer [[id(0)]];
};

struct spvDescriptorSetBuffer1
{
    texture2d<float> tex [[id(0)]];
};

struct main0_out
{
    float2 uv [[user(locn0)]];
    float4 color [[user(locn1)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    float2 pos [[attribute(0)]];
    float2 in_uv [[attribute(1)]];
    float4 in_color [[attribute(2)]];
};

static inline __attribute__((always_inline))
float3 srgb_to_linear(thread const float3& srgb)
{
    bool3 cutoff = srgb < float3(0.040449999272823333740234375);
    float3 higher = pow((srgb + float3(0.054999999701976776123046875)) / float3(1.05499994754791259765625), float3(2.400000095367431640625));
    float3 lower = srgb / float3(12.9200000762939453125);
    return select(higher, lower, cutoff);
}

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]], constant spvDescriptorSetBuffer1& spvDescriptorSet1 [[buffer(1)]])
{
    constexpr sampler smp(filter::linear, mip_filter::linear, compare_func::never, max_anisotropy(1));
    main0_out out = {};
    out.uv = in.in_uv;
    float3 param = float3(in.in_color.xyz);
    out.color = float4(srgb_to_linear(param), in.in_color.w);
    out.gl_Position = (*spvDescriptorSet0.uniform_buffer).mvp * float4(in.pos.x, in.pos.y, 0.0, 1.0);
    return out;
}


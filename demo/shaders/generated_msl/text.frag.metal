#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct PerViewUbo
{
    float4x4 view_proj;
};

struct spvDescriptorSetBuffer0
{
    texture2d<float> tex [[id(0)]];
};

struct main0_out
{
    float4 Target0 [[color(0)]];
};

struct main0_in
{
    float2 f_tex_pos [[user(locn0)]];
    float4 f_color [[user(locn1)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    constexpr sampler smp(filter::linear, mip_filter::linear, address::repeat, compare_func::never, max_anisotropy(1));
    main0_out out = {};
    float alpha = spvDescriptorSet0.tex.sample(smp, in.f_tex_pos).x;
    if (alpha <= 0.0)
    {
        discard_fragment();
    }
    out.Target0 = in.f_color * float4(1.0, 1.0, 1.0, alpha);
    return out;
}


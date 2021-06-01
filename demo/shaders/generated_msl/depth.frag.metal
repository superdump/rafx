#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct PerViewData
{
    float4x4 view;
    float4x4 view_proj;
    float4x4 inv_view3;
};

struct PerObjectData
{
    float4x4 model;
    float4x4 inv_trans_model3;
};

struct spvDescriptorSetBuffer0
{
    constant PerViewData* per_view_data [[id(0)]];
};

struct spvDescriptorSetBuffer2
{
    constant PerObjectData* per_object_data [[id(0)]];
};

struct main0_out
{
    float4 out_normal_vs [[color(0)]];
};

struct main0_in
{
    float3 in_normal_vs [[user(locn0)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]], constant spvDescriptorSetBuffer2& spvDescriptorSet2 [[buffer(2)]])
{
    constexpr sampler smp(filter::linear, mip_filter::linear, address::repeat, compare_func::never, max_anisotropy(16));
    main0_out out = {};
    float3 normal_vs = normalize(float4(in.in_normal_vs, 0.0)).xyz;
    out.out_normal_vs = float4(normal_vs, 1.0);
    return out;
}


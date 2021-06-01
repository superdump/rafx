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
    float3 out_normal_vs [[user(locn0)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    float3 in_pos [[attribute(0)]];
    float3 in_normal [[attribute(1)]];
};

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]], constant spvDescriptorSetBuffer2& spvDescriptorSet2 [[buffer(2)]])
{
    constexpr sampler smp(filter::linear, mip_filter::linear, address::repeat, compare_func::never, max_anisotropy(16));
    main0_out out = {};
    float3 normal_vs = normalize((float3x3((*spvDescriptorSet0.per_view_data).inv_view3[0].xyz, (*spvDescriptorSet0.per_view_data).inv_view3[1].xyz, (*spvDescriptorSet0.per_view_data).inv_view3[2].xyz) * float3x3((*spvDescriptorSet2.per_object_data).inv_trans_model3[0].xyz, (*spvDescriptorSet2.per_object_data).inv_trans_model3[1].xyz, (*spvDescriptorSet2.per_object_data).inv_trans_model3[2].xyz)) * in.in_normal);
    out.out_normal_vs = (normal_vs * 0.5) + float3(0.5);
    out.gl_Position = ((*spvDescriptorSet0.per_view_data).view_proj * (*spvDescriptorSet2.per_object_data).model) * float4(in.in_pos, 1.0);
    return out;
}


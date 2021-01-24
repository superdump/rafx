#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct PerObjectData
{
    float4x4 model;
    float4x4 model_view;
    float4x4 model_view_proj;
};

struct spvDescriptorSetBuffer2
{
    constant PerObjectData* per_object_data [[id(0)]];
};

struct main0_out
{
    float3 out_position_vs [[user(locn0)]];
    float3 out_normal_vs [[user(locn1)]];
    float3 out_tangent_vs [[user(locn2)]];
    float3 out_binormal_vs [[user(locn3)]];
    float2 out_uv [[user(locn4)]];
    float4 out_position_ws [[user(locn5)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    float3 in_pos [[attribute(0)]];
    float3 in_normal [[attribute(1)]];
    float4 in_tangent [[attribute(2)]];
    float2 in_uv [[attribute(3)]];
};

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer2& spvDescriptorSet2 [[buffer(2)]])
{
    main0_out out = {};
    out.gl_Position = (*spvDescriptorSet2.per_object_data).model_view_proj * float4(in.in_pos, 1.0);
    out.out_position_vs = ((*spvDescriptorSet2.per_object_data).model_view * float4(in.in_pos, 1.0)).xyz;
    out.out_normal_vs = float3x3((*spvDescriptorSet2.per_object_data).model_view[0].xyz, (*spvDescriptorSet2.per_object_data).model_view[1].xyz, (*spvDescriptorSet2.per_object_data).model_view[2].xyz) * in.in_normal;
    out.out_tangent_vs = float3x3((*spvDescriptorSet2.per_object_data).model_view[0].xyz, (*spvDescriptorSet2.per_object_data).model_view[1].xyz, (*spvDescriptorSet2.per_object_data).model_view[2].xyz) * in.in_tangent.xyz;
    float3 binormal = cross(in.in_normal, in.in_tangent.xyz) * in.in_tangent.w;
    out.out_binormal_vs = float3x3((*spvDescriptorSet2.per_object_data).model_view[0].xyz, (*spvDescriptorSet2.per_object_data).model_view[1].xyz, (*spvDescriptorSet2.per_object_data).model_view[2].xyz) * binormal;
    out.out_uv = in.in_uv;
    out.out_position_ws = (*spvDescriptorSet2.per_object_data).model * float4(in.in_pos, 1.0);
    return out;
}


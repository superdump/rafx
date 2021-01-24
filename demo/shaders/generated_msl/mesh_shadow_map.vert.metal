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
    float4 gl_Position [[position]];
};

struct main0_in
{
    float3 in_pos [[attribute(0)]];
};

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer2& spvDescriptorSet2 [[buffer(2)]])
{
    main0_out out = {};
    out.gl_Position = (*spvDescriptorSet2.per_object_data).model_view_proj * float4(in.in_pos, 1.0);
    return out;
}


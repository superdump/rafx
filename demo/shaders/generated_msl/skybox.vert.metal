#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct Args
{
    float4x4 mvp;
};

struct spvDescriptorSetBuffer0
{
    constant Args* uniform_buffer [[id(2)]];
};

struct main0_out
{
    float3 out_texcoord [[user(locn0)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    float3 in_pos [[attribute(0)]];
};

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    main0_out out = {};
    float4 pos = (*spvDescriptorSet0.uniform_buffer).mvp * float4(in.in_pos, 1.0);
    out.gl_Position = pos.xyww;
    out.out_texcoord = in.in_pos;
    return out;
}


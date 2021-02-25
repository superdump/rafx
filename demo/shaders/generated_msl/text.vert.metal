#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct PerViewUbo
{
    float4x4 view_proj;
};

struct spvDescriptorSetBuffer1
{
    constant PerViewUbo* per_view_data [[id(0)]];
};

struct main0_out
{
    float2 f_tex_pos [[user(locn0)]];
    float4 f_color [[user(locn1)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    float3 left_top [[attribute(0)]];
    float2 right_bottom [[attribute(1)]];
    float2 tex_left_top [[attribute(2)]];
    float2 tex_right_bottom [[attribute(3)]];
    float4 color [[attribute(4)]];
};

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer1& spvDescriptorSet1 [[buffer(1)]], uint gl_VertexIndex [[vertex_id]])
{
    main0_out out = {};
    float2 pos = float2(0.0);
    float left = in.left_top.x;
    float right = in.right_bottom.x;
    float top = in.left_top.y;
    float bottom = in.right_bottom.y;
    switch (int(gl_VertexIndex))
    {
        case 0:
        {
            pos = float2(left, top);
            out.f_tex_pos = in.tex_left_top;
            break;
        }
        case 1:
        {
            pos = float2(right, top);
            out.f_tex_pos = float2(in.tex_right_bottom.x, in.tex_left_top.y);
            break;
        }
        case 2:
        {
            pos = float2(left, bottom);
            out.f_tex_pos = float2(in.tex_left_top.x, in.tex_right_bottom.y);
            break;
        }
        case 3:
        {
            pos = float2(right, bottom);
            out.f_tex_pos = in.tex_right_bottom;
            break;
        }
    }
    out.f_color = in.color;
    out.gl_Position = (*spvDescriptorSet1.per_view_data).view_proj * float4(pos, in.left_top.z, 1.0);
    return out;
}


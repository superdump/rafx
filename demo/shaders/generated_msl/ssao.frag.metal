#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct PerViewData
{
    float4x4 proj;
    float4x4 inv_proj;
};

struct SsaoConfigUbo
{
    float3 kernel0[32];
    uint kernel_size;
    float radius_vs;
    float bias0;
};

struct spvDescriptorSetBuffer0
{
    constant PerViewData* per_view_data [[id(0)]];
};

struct spvDescriptorSetBuffer1
{
    constant SsaoConfigUbo* ssao_config [[id(0)]];
    texture2d<float> depth_texture [[id(2)]];
    texture2d<float> normal_texture [[id(3)]];
};

struct main0_out
{
    float out_ao [[color(0)]];
};

struct main0_in
{
    float2 in_uv [[user(locn0)]];
};

static inline __attribute__((always_inline))
float rand(thread const float2& co)
{
    return fract(sin(dot(co, float2(12.98980045318603515625, 78.233001708984375))) * 43758.546875);
}

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]], constant spvDescriptorSetBuffer1& spvDescriptorSet1 [[buffer(1)]])
{
    constexpr sampler clamp_sampler(filter::linear, mip_filter::linear, compare_func::never, max_anisotropy(1));
    main0_out out = {};
    int2 frame_size = int2(spvDescriptorSet1.normal_texture.get_width(), spvDescriptorSet1.normal_texture.get_height());
    float frag_depth_ndc = spvDescriptorSet1.depth_texture.sample(clamp_sampler, in.in_uv).x;
    if (frag_depth_ndc == 0.0)
    {
        out.out_ao = 0.5;
        return out;
    }
    float4 frag_vs = (*spvDescriptorSet0.per_view_data).inv_proj * float4((in.in_uv.x * 2.0) - 1.0, 1.0 - (2.0 * in.in_uv.y), frag_depth_ndc, 1.0);
    float3 _97 = frag_vs.xyz / float3(frag_vs.w);
    frag_vs = float4(_97.x, _97.y, _97.z, frag_vs.w);
    float3 normal_vs = (spvDescriptorSet1.normal_texture.sample(clamp_sampler, in.in_uv).xyz - float3(0.5)) * 2.0;
    float2 param = in.in_uv;
    float2 param_1 = in.in_uv + float2(1.0);
    float3 random_vec = normalize(float3(rand(param), rand(param_1), 0.0));
    float3 tangent_vs = normalize(random_vec - (normal_vs * dot(random_vec, normal_vs)));
    float3 bitangent_vs = cross(normal_vs, tangent_vs);
    float3x3 TBN = float3x3(float3(tangent_vs), float3(bitangent_vs), float3(normal_vs));
    float occlusion = 0.0;
    for (int i = 0; uint(i) < (*spvDescriptorSet1.ssao_config).kernel_size; i++)
    {
        float3 sample_offset_vs = TBN * (*spvDescriptorSet1.ssao_config).kernel0[i];
        float4 sample_vs = float4(frag_vs.xyz + (sample_offset_vs * (*spvDescriptorSet1.ssao_config).radius_vs), 1.0);
        float4 sample_cs = (*spvDescriptorSet0.per_view_data).proj * sample_vs;
        float3 sample_ndc = sample_cs.xyz / float3(sample_cs.w);
        float2 depth_uv = float2((sample_ndc.x * 0.5) + 0.5, (sample_ndc.y * (-0.5)) + 0.5);
        float sample_depth_ndc = spvDescriptorSet1.depth_texture.sample(clamp_sampler, depth_uv).x;
        float4 sample_depth_vs = (*spvDescriptorSet0.per_view_data).inv_proj * float4(0.0, 0.0, sample_depth_ndc, 1.0);
        float3 _235 = sample_depth_vs.xyz / float3(sample_depth_vs.w);
        sample_depth_vs = float4(_235.x, _235.y, _235.z, sample_depth_vs.w);
        float range_check = smoothstep(0.0, 1.0, (*spvDescriptorSet1.ssao_config).radius_vs / abs(frag_vs.z - sample_depth_vs.z));
        occlusion += (float(sample_depth_vs.z >= (sample_vs.z + (*spvDescriptorSet1.ssao_config).bias0)) * range_check);
    }
    occlusion = 1.0 - (occlusion / float((*spvDescriptorSet1.ssao_config).kernel_size));
    out.out_ao = occlusion;
    return out;
}


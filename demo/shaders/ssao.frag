#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "ssao_textures.glsl"
#include "ssao_uniform.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out float out_ao;

float rand(vec2 co) {
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    // tile noise texture over screen, based on screen dimensions divided by noise size
    const ivec2 frame_size = textureSize(sampler2D(normal_texture, clamp_sampler), 0);
    const ivec2 noise_size = textureSize(sampler2D(noise_texture, repeat_sampler), 0);
    const vec2 noise_scale = vec2(frame_size) / vec2(noise_size);

    // Calculate the fragment position from the depth texture
    float frag_depth_ndc = texture(sampler2D(depth_texture, clamp_sampler), in_uv).x;
    if (frag_depth_ndc == 1.0) {
        out_ao = 1.0;
        return;
    }
    // in_uv.x is [0,1] from left to right. * 2 - 1 remaps to [-1, 1] left to right which is NDC
    // in_uv.y is [0,1] top to bottom. (1-v)*2-1 = 2-2v-1 = 1-2v remaps to [-1, 1] bottom to top which is NDC
    vec4 frag_vs = per_view_data.inv_proj * vec4(in_uv.x * 2.0 - 1.0, 1.0 - 2.0 * in_uv.y, frag_depth_ndc, 1.0);
    frag_vs.xyz /= frag_vs.w;
    vec3 normal_vs = (texture(sampler2D(normal_texture, clamp_sampler), in_uv).xyz - 0.5) * 2.0;
    vec3 random_vec = texture(sampler2D(noise_texture, repeat_sampler), in_uv * noise_scale).xyz;

    vec3 tangent_vs = normalize(random_vec - normal_vs * dot(random_vec, normal_vs));
    vec3 bitangent_vs = cross(normal_vs, tangent_vs);
    mat3 TBN = mat3(tangent_vs, bitangent_vs, normal_vs);

    float occlusion = 0.0;
    for (int i = 0; i < ssao_config.kernel_size; ++i) {
        vec3 sample_offset_vs = TBN * ssao_config.kernel[i].xyz; // from tangent to view space
        vec4 sample_vs = vec4(frag_vs.xyz + sample_offset_vs * ssao_config.radius_vs, 1.0);
        vec4 sample_cs = per_view_data.proj * sample_vs; // from view to clip space
        vec3 sample_ndc = sample_cs.xyz / sample_cs.w; // perspective divide
        // sample_ndc.x is [-1,1] left to right, so * 0.5 + 0.5 remaps to [0,1] left to right
        // sample_ndc.y is [-1,1] bottom to top, so * -0.5 + 0.5 remaps to [0,1] top to bottom
        vec2 depth_uv = vec2(sample_ndc.x * 0.5 + 0.5, sample_ndc.y * -0.5 + 0.5);

        float sample_depth_ndc = texture(sampler2D(depth_texture, clamp_sampler), depth_uv).x;
        vec4 sample_depth_vs = per_view_data.inv_proj * vec4(0.0, 0.0, sample_depth_ndc, 1.0);
        sample_depth_vs.xyz /= sample_depth_vs.w;

        float range_check = smoothstep(0.0, 1.0, ssao_config.radius_vs / abs(frag_vs.z - sample_depth_vs.z));
        occlusion += (sample_depth_vs.z >= sample_vs.z + ssao_config.bias ? 1.0 : 0.0) * range_check;
    }
    occlusion = 1.0 - (occlusion / ssao_config.kernel_size);

    out_ao = occlusion;
}

#version 450

#include "depth_textures.glsl"
#include "depth_uniform.glsl"

layout(location = 0) in vec3 in_normal_vs;

#ifdef PBR_TEXTURES
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_tangent_vs;
layout(location = 3) in vec4 in_binormal_vs;
#endif

layout (location = 0) out vec4 out_normal_vs;

// #ifdef PBR_TEXTURES
// // Passing texture/sampler through like this breaks reflection metadata so for now just grab global data
// vec4 normal_map(
//     mat3 tangent_binormal_normal,
//     vec2 uv
// ) {
//     // Sample the normal and unflatten it from the texture (i.e. convert
//     // range of [0, 1] to [-1, 1])
//     vec3 normal = texture(sampler2D(normal_texture, smp), uv).xyz;
//     normal = normal * 2.0 - 1.0;

//     // Transform the normal from the texture with the TNB matrix, which will put
//     // it into the TNB's space (view space))
//     normal = tangent_binormal_normal * normal;
//     return normalize(vec4(normal, 0.0));
// }
// #endif

void main() {
    // Calculate the normal (use the normal map if it exists)
    vec3 normal_vs;

#ifdef PBR_TEXTURES
    // if (per_material_data.data.has_normal_texture) {
    //     mat3 tbn = mat3(in_tangent_vs, in_binormal_vs, in_normal_vs);
    //     normal_vs = normal_map(
    //         tbn,
    //         in_uv
    //     ).xyz;
    // } else {
        normal_vs = normalize(vec4(in_normal_vs, 0)).xyz;
    // }
#else
    normal_vs = normalize(vec4(in_normal_vs, 0)).xyz;
#endif

    out_normal_vs = vec4(normal_vs, 1.0);
}

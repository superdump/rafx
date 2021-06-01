#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

#include "depth_uniform.glsl"

// @[semantic("POSITION")]
layout (location = 0) in vec3 in_pos;
// @[semantic("NORMAL")]
layout (location = 1) in vec3 in_normal;

#ifdef PBR_TEXTURES
// @[semantic("TEXCOORDS")]
layout (location = 2) in vec2 in_uv;
// @[semantic("TANGENT")]
layout (location = 3) in vec4 in_tangent;
#endif

layout (location = 0) out vec3 out_normal_vs;

#ifdef PBR_TEXTURES
layout (location = 1) out vec2 out_uv;
layout (location = 2) out vec4 out_tangent_vs;
layout (location = 3) out vec3 out_bitangent_vs;
#endif

void main() {
    // For non-uniform scaling, the model matrix must be inverse-transposed which has the effect of
    // applying inverse scaling while retaining the correct rotation
    // The normals must be re-normalised after applying the inverse-transpose because this can affect
    // the length of the normal
    // The normals need to rotate inverse to the view rotation
    // Using mat3 is important else the translation in the model matrix can have other unintended effects
    vec3 normal_vs = normalize(mat3(per_view_data.inv_view3) * mat3(per_object_data.inv_trans_model3) * in_normal);
    out_normal_vs = normal_vs * 0.5 + 0.5;

#ifdef PBR_TEXTURES
    out_uv = in_uv;

    mat3 model_view = mat3(per_view_data.view) * mat3(per_object_data.model);
    vec4 tangent_vs = vec4(normalize(model_view * in_tangent.xyz), in_tangent.w);
    out_tangent_vs = tangent_vs;

    out_bitangent_vs = cross(normal_vs, tangent_vs.xyz) * tangent_vs.w;
#endif

    gl_Position = per_view_data.view_proj * per_object_data.model * vec4(in_pos, 1.0);
}

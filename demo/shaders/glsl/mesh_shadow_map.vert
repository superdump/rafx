#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

// set = 2, binding = 0
#include "mesh_common_bindings.glsl"

// @[semantic("POSITION")]
layout (location = 0) in vec3 in_pos;

void main() {
    //vec4 temp = per_object_data.model_view_proj * vec4(in_pos, 1.0);
    //gl_Position = temp / temp.w;

    //gl_Position = vec4(0.0, 0.0, 0.0, 1.0);

    gl_Position = per_object_data.model_view_proj * vec4(in_pos, 1.0);
}

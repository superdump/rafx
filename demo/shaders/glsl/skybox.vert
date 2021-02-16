#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "skybox.glsl"

// @[semantic("POSITION")]
layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec3 out_texcoord;

void main() {
    vec4 pos = uniform_buffer.mvp * vec4(in_pos, 1.0);
    // Use perspective divide to force z to be 1.0, which means it will draw behind everything
    gl_Position = pos.xyww;
    out_texcoord = in_pos;
}

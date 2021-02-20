#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "skybox.glsl"

layout(location = 0) out vec3 out_texcoord;

void main() {

    gl_Position = vec4(((gl_VertexIndex << 1) & 2) * 2.0 - 1.0, (gl_VertexIndex & 2) * 2.0 - 1.0, 0.0, 1.0);
    out_texcoord = (uniform_buffer.inverse_view * uniform_buffer.inverse_projection * gl_Position).xyz;
    //gl_Position = vec4(outUV * 2.0f - 1.0f, 0.0f, 1.0f);

/*
    vec4 pos = uniform_buffer.mvp * vec4(in_pos, 1.0);
    // Use perspective divide to force z to be 1.0, which means it will draw behind everything
    gl_Position = pos.xyww;
    out_texcoord = in_pos;
*/
}

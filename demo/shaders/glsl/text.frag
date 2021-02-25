#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "text.glsl"

// Based on example from brush_glyph, which is apache license 2.0

layout(location = 0) in vec2 f_tex_pos;
layout(location = 1) in vec4 f_color;

layout(location = 0) out vec4 Target0;

void main() {
    float alpha = texture(sampler2D(tex, smp), f_tex_pos).r;
    if (alpha <= 0.0) {
        discard;
    }
    Target0 = f_color * vec4(1.0, 1.0, 1.0, alpha);
}

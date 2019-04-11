#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 frag_color;

layout(set = 1, binding = 1) uniform sampler2D tex_sampler;

void main() {
    frag_color = texture(tex_sampler, tex);
}

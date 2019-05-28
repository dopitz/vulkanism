#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 frag_color;

layout(set = 1, binding = 2) uniform sampler2D tex_sampler;

void main() {
  frag_color = vec4(1,1,1, texture(tex_sampler, tex).r);
}

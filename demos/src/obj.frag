#version 450
#extension GL_ARB_separate_shader_objects : enable


layout(location = 0) in vec3 vs_position;
layout(location = 1) in vec3 vs_normal;
layout(location = 2) in vec2 tex;

layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = vec4(vs_normal, 1);
    //frag_color = vec4(vs_position, 1);
}




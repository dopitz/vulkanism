#version 450
#extension GL_ARB_separate_shader_objects : enable

//layout(location = 0) in vec2 tex;

layout(location = 0) out vec4 frag_color;

//layout(binding = 0) uniform sampler2D texSampler;

void main() {
    //frag_color = texture(texSampler, tex);
    frag_color = vec4(1, 0, 0, 1);
}



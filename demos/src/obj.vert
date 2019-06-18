#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 position;
layout(location = 1) in vec2 tex;

layout(location = 0) out vec2 out_tex;

layout(binding = 0) uniform ub_transform {
  mat4 model;
  mat4 view;
  mat4 proj;
};

void main() {
  gl_Position = proj * view * model * position;
  out_tex = tex;
  //gl_Position = position;
}


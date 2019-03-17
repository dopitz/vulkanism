#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 position;

//layout(location = 0) out vec2 tex;

layout(binding = 0) uniform ub_transform {
  mat4 model;
  mat4 view;
  mat4 proj;
};

void main() {
  gl_Position = proj * view * model * position;
  //gl_Position = position;
}


#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex;

layout(location = 0) out vec3 vs_position;
layout(location = 1) out vec3 vs_normal;
layout(location = 2) out vec2 out_tex;

layout(binding = 0) uniform ub_transform {
  mat4 model;
  mat4 view;
  mat4 proj;
};

void main() {
  gl_Position = proj * view * model * vec4(position, 1);
  vs_position = (view * model * vec4(position, 1)).xyz;
  vs_normal = (view * model * vec4(normal, 0)).xyz;
  out_tex = tex;
  //gl_Position = position;
}


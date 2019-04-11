#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in uvec2 position;
layout(location = 1) in uvec2 size;

layout(location = 0) out vec2 out_tex;

layout(set = 0, binding = 0) uniform ub_viewport {
  uint width;
  uint height;
};

vec2 positions[4] = vec2[](
  vec2(-1, -1), 
  vec2(-1, 1),
  vec2(1, -1),
  vec2(1, 1)
);

void main() {
  vec2 vp = vec2(width, height);
  vec2 pos = (0.5 * positions[gl_VertexIndex] + 0.5) / vp * size - vec2(1) + position / vp;

  gl_Position = vec4(pos, 0, 1);
  out_tex = vec2(0,0);
}
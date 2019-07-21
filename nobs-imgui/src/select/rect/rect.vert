#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 size;

layout(location = 0) out flat uint out_id;

layout(set = 0, binding = 0) uniform ub_viewport {
  uint width;
  uint height;
};

layout(set = 1, binding = 1) uniform ub {
  uint id_offset;
};

vec2 positions[4] = vec2[](
  vec2(-1, -1), 
  vec2(-1, 1),
  vec2(1, -1),
  vec2(1, 1)
);

void main() {
  vec2 vp = vec2(width, height);
  vec2 pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * size + position * 2 / vp - vec2(1);

  gl_Position = vec4(pos, 0, 1);

  out_id = id_offset + gl_VertexIndex;
}


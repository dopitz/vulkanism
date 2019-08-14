#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) out flat uint id;

layout(set = 0, binding = 0) uniform ub_viewport {
  uint width;
  uint height;
};

layout(set = 1, binding = 1) uniform ub {
  ivec2 position;
  ivec2 size;
  ivec2 bd_thickness;
  uint id_body;
  uint id_border;
};

vec2 positions[12] = vec2[](
  vec2(-1, -1), 
  vec2(-1, 1),
  vec2(1, -1),
  vec2(1, 1),

  vec2(1, 1),
  vec2(-1, 1),
  vec2(-1, 1),
  vec2(-1, -1),
  vec2(-1, -1),
  vec2(1, -1),
  vec2(1, -1),
  vec2(1, 1)
);

void main() {
  vec2 vp = vec2(width, height);
  vec2 pos = vec2(0,0);
  
  if (gl_VertexIndex < 11 && (gl_VertexIndex % 2 == 1 || gl_VertexIndex < 4)) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * (size - 2 * bd_thickness) + (position + bd_thickness) * 2 / vp - vec2(1);
  }
  else {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * size + position * 2 / vp - vec2(1);
  }

  // smaller 3 works because id is interpolated with flat
  if (gl_VertexIndex < 3) id = id_body;
  else id = id_border;

  gl_Position = vec4(pos, 0, 1);
}

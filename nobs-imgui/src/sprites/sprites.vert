#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 size;
layout(location = 2) in vec2 tex_bl;
layout(location = 3) in vec2 tex_tr;

layout(location = 0) out vec2 out_tex;

layout(set = 0, binding = 0) uniform ub_viewport {
  uint width;
  uint height;
};

layout(set = 1, binding = 1) uniform ub {
  ivec2 offset;
};

vec2 positions[4] = vec2[](
  vec2(-1, -1), 
  vec2(-1, 1),
  vec2(1, -1),
  vec2(1, 1)
);

void main() {
  vec2 vp = vec2(width, height);
  vec2 pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * size + offset * 2 / vp + position * 2 / vp - vec2(1);

  gl_Position = vec4(pos, 0, 1);

  out_tex = vec2(0,0);
  if (gl_VertexIndex == 0) out_tex = vec2(tex_bl.x, tex_tr.y);
  else if (gl_VertexIndex == 1) out_tex = tex_bl;
  else if (gl_VertexIndex == 2) out_tex = tex_tr;
  else if (gl_VertexIndex == 3) out_tex = vec2(tex_tr.x, tex_bl.y);
}

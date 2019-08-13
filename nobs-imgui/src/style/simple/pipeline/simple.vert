#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform ub_viewport {
  uint width;
  uint height;
};

layout(set = 1, binding = 1) uniform ub {
  ivec2 pos;
  ivec2 size;
  ivec2 bd_thickness;
  vec2 bd_texcoord;
};

vec2 positions[12] = vec2[](
  vec2(-1, -1), 
  vec2(-1, 1),
  vec2(1, -1),
  vec2(1, 1),

  vec2(1, 1), 
  vec2(1, 1), 
  vec2(1, 1), 
  vec2(1, 1), 
  vec2(1, 1), 
  vec2(1, 1), 
  vec2(1, 1), 
  vec2(1, 1)
);

void main() {
  vec2 vp = vec2(width, height);
  vec2 p = vec2(0, 0);

  if (gl_VertexIndex % 2 == 0 || gl_VertexIndex < 4) {
    p = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * (size - bd_thickness) + pos * 2 / vp - vec2(1);
    color = vec4(1,0,0,1);
  }
  else {
    p = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * size + pos * 2 / vp - vec2(1);
    color = vec4(0,1,0,1);
  }


  gl_Position = vec4(pos, 0, 1);

  //if (gl_VertexIndex == 0) out_tex = vec2(tex_bl.x, tex_tr.y);
  //else if (gl_VertexIndex == 1) out_tex = tex_bl;
  //else if (gl_VertexIndex == 2) out_tex = tex_tr;
  //else if (gl_VertexIndex == 3) out_tex = vec2(tex_tr.x, tex_bl.y);
}


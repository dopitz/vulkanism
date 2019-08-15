layout(set = 0, binding = 0) uniform ub_viewport {
  uint width;
  uint height;
};

layout(set = 1, binding = 1) uniform ub_style_lut {
  vec4 color_body;
  vec4 color_bd_inner;
  vec4 color_bd_outer;
  ivec2 bd_thickness;
};

layout(set = 2, binding = 2) uniform ub {
  ivec2 position;
  ivec2 size;
  uint id_body;
  uint id_bd_topleft;
  uint id_bd_topright;
  uint id_bd_bottomleft;
  uint id_bd_bottomright;
  uint id_bd_top;
  uint id_bd_bottom;
  uint id_bd_left;
  uint id_bd_right;
};

vec2 positions[6] = vec2[](
  vec2(-1, -1), 
  vec2(-1, 1),
  vec2(1, -1),

  vec2(1, -1),
  vec2(-1, 1),
  vec2(1, 1)
);

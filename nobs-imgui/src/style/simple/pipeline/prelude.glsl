layout(set = 0, binding = 0) uniform ub_viewport {
  uint width;
  uint height;
};

layout(set = 1, binding = 1) uniform ub_style {
  vec4 color_body;
  vec4 color_bd_inner;
  vec4 color_bd_outer;
  ivec2 bd_thickness;
};

layout(set = 2, binding = 2) uniform ub {
  ivec2 position;
  ivec2 size;
  uint id_body;
};

vec2 positions[6] = vec2[](
  vec2(-1, -1), 
  vec2(-1, 1),
  vec2(1, -1),

  vec2(1, -1),
  vec2(-1, 1),
  vec2(1, 1)
);

#define ID_BODY         id_body
#define ID_TOPLEFT      id_body + 1
#define ID_TOPRIGHT     id_body + 2
#define ID_BOTTOMLEFT   id_body + 3
#define ID_BOTTOMRIGHT  id_body + 4
#define ID_TOP          id_body + 5
#define ID_BOTTOM       id_body + 6
#define ID_LEFT         id_body + 7
#define ID_RIGHT        id_body + 8

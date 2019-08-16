#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) out flat uint id;

#include "prelude.glsl"

void main() {
  vec2 vp = vec2(width, height);
  vec2 pos = vec2(0,0);

  if (gl_InstanceIndex == 0) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * (size - 2 * bd_thickness) + (position + bd_thickness) * 2 / vp - vec2(1);
    id = ID_BODY;
  }
  else if (gl_InstanceIndex == 1) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * bd_thickness + position * 2 / vp - vec2(1);
    id = ID_TOPLEFT;
  }
  else if (gl_InstanceIndex == 2) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * bd_thickness + (position + ivec2((size - bd_thickness).x, 0)) * 2 / vp - vec2(1);
    id = ID_TOPRIGHT;
  }
  else if (gl_InstanceIndex == 3) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * bd_thickness + (position + ivec2(0, (size - bd_thickness).y)) * 2 / vp - vec2(1);
    id = ID_BOTTOMLEFT;
  }
  else if (gl_InstanceIndex == 4) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * bd_thickness + (position + size - bd_thickness) * 2 / vp - vec2(1);
    id = ID_BOTTOMRIGHT;
  }
  else if (gl_InstanceIndex == 5) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * ivec2(size.x - 2 * bd_thickness.x, bd_thickness.y) + (position + vec2(bd_thickness.x, 0)) * 2 / vp - vec2(1);
    id = ID_TOP;
  }
  else if (gl_InstanceIndex == 6) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * ivec2(size.x - 2 * bd_thickness.x, bd_thickness.y) + (position + vec2(bd_thickness.x, size.y - bd_thickness.y)) * 2 / vp - vec2(1);
    id = ID_BOTTOM;
  }
  else if (gl_InstanceIndex == 7) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * ivec2(bd_thickness.x, size.y - 2 * bd_thickness.y) + (position + vec2(0, bd_thickness.y)) * 2 / vp - vec2(1);
    id = ID_LEFT;
  }
  else if (gl_InstanceIndex == 8) {
    pos = (0.5 * positions[gl_VertexIndex] + 0.5) * 2 / vp * ivec2(bd_thickness.x, size.y - 2 * bd_thickness.y) + (position + vec2(size.x - bd_thickness.x, bd_thickness.y)) * 2 / vp - vec2(1);
    id = ID_RIGHT;
  }

  gl_Position = vec4(pos, 0, 1);
}

#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in flat uint id;
layout(location = 0) out uint frag_id;

void main() {
  frag_id = id;
}

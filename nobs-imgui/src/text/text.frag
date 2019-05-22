#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 frag_color;

layout(set = 1, binding = 2) uniform sampler2D tex_sampler;

void main() {
  frag_color = vec4(texture(tex_sampler, tex).rrr,1);
  //frag_color = vec4(1,1,1,1);

  //float d = texture(tex_sampler, tex).r;
  ////if (d < 0.49) discard;
  ////frag_color = vec4(1,1,1,1);
  //if (d < 0.35) {
  //  discard;
  //}
  //if (d < 0.55) {
  //  float v = smoothstep(0, 1, (d - 0.25) / (0.2));
  //  frag_color = vec4(v,v,v, 1);
  //}
  //else {
  //  frag_color = vec4(1,1,1,1);
  //}
}

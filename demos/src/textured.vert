#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 position;

//layout(location = 0) out vec2 tex;
out gl_PerVertex {
  vec4 gl_Position;
};

void main() {
  gl_Position = vec4(position.xy, 0.0, 1.0);
}


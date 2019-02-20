#version 450
#extension GL_ARB_separate_shader_objects : enable

//layout(location = 0) out vec2 tex;
out gl_PerVertex {
  vec4 gl_Position;
};

vec2 positions[3] = vec2[](
  vec2(0.0, -0.5), 
  vec2(-0.5, 0.5),
  vec2(0.5, 0.5)
);


//vec2 positions[3] = vec2[](
//  vec2(-1, -1), 
//  vec2(-1, 3),
//  vec2(3, -1)
//);

void main() {
  gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}

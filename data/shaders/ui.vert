#version 450 core

layout (location = 0) in vec2 vsInPos;
layout (location = 1) in vec2 vsInUV;
layout (location = 2) in vec4 vsInColor;

out gl_PerVertex {
  vec4 gl_Position;
};

out VS_OUT_FS_IN {
  vec2 uv;
  vec4 color;
} vs_out_fs_in;

layout (location = 0) uniform mat4 TransformMatrix;

void main(void) {
  gl_Position = TransformMatrix * vec4(vsInPos, 0.0, 1.0);
  vs_out_fs_in.uv = vsInUV;
  vs_out_fs_in.color = vsInColor;
}

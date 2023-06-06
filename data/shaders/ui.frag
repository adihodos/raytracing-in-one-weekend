#version 450 core

in VS_OUT_FS_IN {
  vec2 uv;
  vec4 color;
} fs_in;

layout (binding = 0) uniform sampler2D TexAtlas;

out vec4 FinalFragColor;

void main(void) {
  FinalFragColor = fs_in.color * texture(TexAtlas, fs_in.uv).rrrr;
}

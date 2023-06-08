#version 460 core

layout(location = 1) out vec2 texCoord;

out gl_PerVertex {
    layout(location = 0) vec4 gl_Position;
};

void main()
{
    vec2 position = vec2(gl_VertexID % 2, gl_VertexID / 2) * 4.0 - 1;
    texCoord = (position + 1) * 0.5;
    texCoord.y = 1.0 - texCoord.y;

    gl_Position = vec4(position, 0, 1);
}

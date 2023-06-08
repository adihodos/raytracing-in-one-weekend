#version 460 core

layout(location = 1) in vec2 texCoord;
layout(binding = 0) uniform sampler2D texImg;
layout(location = 0) out vec4 FinalFragColor;

void main()
{
    FinalFragColor = texture(texImg, texCoord);
}

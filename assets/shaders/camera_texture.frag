#version 330 core
out vec4 FragColor;

in vec2 TexCoord;

//in vec3 color;
uniform vec3 color;

// texture sampler
uniform sampler2D texture1;

void main()
{
    FragColor = texture(texture1, TexCoord);
}
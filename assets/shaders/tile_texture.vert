#version 330 core
layout (location = 0) in vec3 inPostion;
layout (location = 1) in vec2 inTexCoord;

out vec2 TexCoord;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform vec2 tex_size;
uniform vec2 offset;

void main()
{
    gl_Position = projection * view * model * vec4(inPostion, 1.0f);

    TexCoord = vec2((inTexCoord.x + offset.x) / tex_size.x, (inTexCoord.y + offset.y) / tex_size.y);
}
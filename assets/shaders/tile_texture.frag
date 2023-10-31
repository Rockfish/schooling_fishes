#version 330 core
out vec4 FragColor;

in vec2 TexCoord;

//uniform vec3 color;
uniform sampler2D texture_diffuse1;

uniform vec2 offset;

void main()
{
    float fwidth = 384.0;
    float fheight = 256.0;

//    float x_offset = 32.0;
//    float y_offset = 0.0;

    vec2 tex_coord = vec2((TexCoord.x + offset.x) / fwidth, (TexCoord.y + offset.y) / fheight);

    vec4 texColor = texture(texture_diffuse1, tex_coord);

    if(texColor.a < 0.1)
        discard;

    FragColor = texColor;
}
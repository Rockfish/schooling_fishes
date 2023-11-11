#version 330 core
out vec4 FragColor;

in vec2 TexCoord;

//uniform vec3 color;
uniform sampler2D texture_diffuse1;

uniform float alpha;



void main()
{
    vec4 texColor = texture(texture_diffuse1, TexCoord);
    vec4 alphatexColor = vec4(texColor.r, texColor.g, texColor.b, alpha);

    if(texColor.a < 0.1)
        discard;

    FragColor = alphatexColor;
}
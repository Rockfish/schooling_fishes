#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform float current_time;


vec3 projectVector(vec3 a, vec3 b) {
    float dotProduct = dot(a, b);
    float magnitudeSquaredB = dot(b, b);
    float scalar = dotProduct / magnitudeSquaredB;
    return b * scalar;
}

void main()
{
    vec3 proj = projectVector(aPos, vec3(10.0, 1.0, 10.0));
    float h = 5.0 * cos((length(proj) + current_time * 10.0) * 0.2);

    vec3 proj_2 = projectVector(aPos, vec3(10.0, 1.0, 3.0));
    h = h + 5.0 * cos((length(proj_2) + current_time * 8.0) * -0.2);

    vec3 proj_3 = projectVector(aPos, vec3(3.0, 1.0, 40.0));
    h = h + 2.0 * cos((length(proj_3) + current_time * 50.0) * -0.1);

    vec3 newPos = vec3(aPos.x, h, aPos.z);

    gl_Position = projection * view * model * vec4(newPos, 1.0f);
    TexCoord = vec2(aTexCoord.x, aTexCoord.y);
}
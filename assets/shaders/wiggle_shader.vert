#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 norm;
layout(location = 2) in vec2 tex;
//layout(location = 3) in vec3 tangent;
//layout(location = 4) in vec3 bitangent;
//layout(location = 5) in ivec4 boneIds;
//layout(location = 6) in vec4 weights;

out vec2 TexCoords;
out vec3 Norm;
out vec4 FragPosLightSpace;
out vec3 FragWorldPos;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform mat4 aimRot;
uniform mat4 lightSpaceMatrix;

uniform vec3 nosePos;
uniform float time;

const float wiggleMagnitude = 0.02;
const float wiggleDistModifier = 2.5;
const float wiggleTimeModifier = 9.4;

float easeInOutCubic(float x) {
  return x < 0.5 ? 4 * x * x * x : 1 - pow(-2 * x + 2, 3) / 2;
}

float easeInQuart(float x) {
  return x * x * x * x;
}

void main() {

  float distance_from_nose =  wiggleDistModifier * distance(nosePos, pos);
  float magnitude = easeInQuart(distance_from_nose) * wiggleMagnitude;

  float xOffset = sin(wiggleTimeModifier * time + distance_from_nose) * magnitude;

  gl_Position = projection * view * model * vec4(pos.x + xOffset, pos.y, pos.z, 1.0);

  TexCoords = tex;

  FragPosLightSpace = lightSpaceMatrix * model * vec4(pos, 1.0);

  // TODO fix norm for wiggle
  Norm = vec3(aimRot * vec4(norm, 1.0));

  FragWorldPos = vec3(model * vec4(pos, 1.0));
}


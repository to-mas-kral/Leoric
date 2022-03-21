#version 460 core

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec2 inTexcoords;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec3 inJoints;
layout(location = 7) in vec3 inWeights;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec2 texCoords;
out vec3 normal;
out vec3 fragPos;

void main() {
    gl_Position = projection * view * model * vec4(inPos, 1.0);

    texCoords = inTexcoords;
    normal = mat3(transpose(inverse(model))) * inNormal;
    fragPos = vec3(model * vec4(inPos, 1.0));
}

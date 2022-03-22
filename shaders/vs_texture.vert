#version 460 core

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec2 inTexcoords;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in uvec4 inJoints;
layout(location = 4) in vec4 inWeights;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform mat4 jointMatrices[64];
uniform uint drawingPoints;

out vec2 texCoords;
out vec3 normal;
out vec3 fragPos;

void main() {
    mat4 modelTransform;

    if (drawingPoints == 1) {
        modelTransform = model;
    } else {
        modelTransform =
            inWeights.x * jointMatrices[inJoints.x] +
            inWeights.y * jointMatrices[inJoints.y] +
            inWeights.z * jointMatrices[inJoints.z] +
            inWeights.w * jointMatrices[inJoints.w];

        modelTransform = model;
    }

    gl_Position = projection * view * modelTransform * vec4(inPos, 1.0);

    texCoords = inTexcoords;
    normal = mat3(transpose(inverse(model))) * inNormal;
    fragPos = vec3(model * vec4(inPos, 1.0));
}

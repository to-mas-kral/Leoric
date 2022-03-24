#version 460 core

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec2 inTexcoords;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in uvec4 inJoints;
layout(location = 4) in vec4 inWeights;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform mat4 jointMatrices[128]; // TODO: Shader Storage Buffer Objects.
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
            (inWeights.x * jointMatrices[int(inJoints.x)]) +
            (inWeights.y * jointMatrices[int(inJoints.y)]) +
            (inWeights.z * jointMatrices[int(inJoints.z)]) +
            (inWeights.w * jointMatrices[int(inJoints.w)]);
    }

    gl_Position = projection * view * modelTransform * vec4(inPos, 1.0);

    if (drawingPoints == 1) {
        gl_Position.z = 0.0;
    }

    texCoords = inTexcoords;

    // FIMXE: light calculations model matrices
    normal = mat3(transpose(inverse(model))) * inNormal;
    fragPos = vec3(model * vec4(inPos, 1.0));
}

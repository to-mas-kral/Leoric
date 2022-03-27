#version 460 core

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec2 inTexcoords;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in uvec4 inJoints;
layout(location = 4) in vec4 inWeights;

layout (std140, binding = 1) uniform Transforms {
    mat4 projection;
    mat4 view;
    mat4 model;
};

layout (std140, binding = 2) uniform JointMatrices {
    mat4 jointMatrices[256];
};

out VsOut {
    vec2 texCoords;
    // vec3 normal;
    // vec3 fragPos;
} vsOut;

void main() {
    // https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#joint-hierarchy
    // "Only the joint transforms are applied to the skinned mesh; the transform of the
    // skinned mesh node MUST be ignored."
    mat4 model =
        (inWeights.x * jointMatrices[int(inJoints.x)]) +
        (inWeights.y * jointMatrices[int(inJoints.y)]) +
        (inWeights.z * jointMatrices[int(inJoints.z)]) +
        (inWeights.w * jointMatrices[int(inJoints.w)]);

    gl_Position = projection * view * model * vec4(inPos, 1.0);

    vsOut.texCoords = inTexcoords;

    // FIMXE: light calculations model matrices
    // normal = mat3(transpose(inverse(model))) * inNormal;
    // fragPos = vec3(model * vec4(inPos, 1.0));
}

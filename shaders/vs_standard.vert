#version 460 core

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec2 inTexcoords;
layout(location = 2) in vec3 inNormal;

layout (std140, binding = 1) uniform Tranforms {
    mat4 projection;
    mat4 view;
    mat4 model;
};

out VsOut {
    vec2 texCoords;
    // vec3 normal;
    // vec3 fragPos;
} vsOut;

void main() {
    gl_Position = projection * view * model * vec4(inPos, 1.0);

    vsOut.texCoords = inTexcoords;

    // FIMXE: light calculations model matrices
    // normal = mat3(transpose(inverse(model))) * inNormal;
    // fragPos = vec3(model * vec4(inPos, 1.0));
}

#version 460 core

layout(location = 0) in vec3 vertexPos;
layout(location = 1) in vec2 vertexTexcoords;
layout(location = 2) in vec3 vertexNormal;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec2 texCoords;
out vec3 normal;
out vec3 fragPos;

void main() {
    gl_Position = projection * view * model * vec4(vertexPos, 1.0);

    texCoords = vertexTexcoords;
    normal = mat3(transpose(inverse(model))) * vertexNormal;
    fragPos = vec3(model * vec4(vertexPos, 1.0));
}

#version 460 core

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec2 vertex_texcoords;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec2 texCoords;

void main() {
    gl_Position = projection * view * model * vec4(vertex_position, 1.0);
    texCoords = vertex_texcoords;
}

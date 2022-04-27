#version 420 core

in VsOut {
    vec2 texCoords;
    vec3 normal;
    vec3 fragPos;
} vsOut;

layout (std140, binding = 4) uniform Material {
    uniform vec4 texBaseColorFactor;
    uniform vec3 lightPos;
};

out vec4 FragColor;

void main() {
    vec4 texColor = texBaseColorFactor;
    FragColor = texColor;
}

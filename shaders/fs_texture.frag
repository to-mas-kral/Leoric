#version 420 core

in VsOut {
    vec2 texCoords;
    vec3 normal;
    vec3 fragPos;
} vsOut;

layout (std140, binding = 4) uniform Material {
    uniform vec4 texBaseColorFactor;
};

uniform sampler2D myTexture;

out vec4 FragColor;

void main() {
    vec4 texColor = texture(myTexture, vsOut.texCoords) * texBaseColorFactor;

    // ambient
    vec4 ambientColor = texColor * 0.1;

    vec3 lightPos = vec3(200, 100, 300);

    vec3 lightDir = normalize(lightPos - vsOut.fragPos);
    vec3 norm = normalize(vsOut.normal);

    // diffuse
    float diffuseK = max(dot(norm, lightDir), 0);
    vec4 diffuseColor = texColor * vec4(vec3(diffuseK), 1.0);

    FragColor = ambientColor + diffuseColor;
}

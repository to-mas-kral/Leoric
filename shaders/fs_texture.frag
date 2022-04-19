#version 420 core

in VsOut {
    vec2 texCoords;
    //vec3 normal;
    //vec3 fragPos;
} vsOut;


//struct Material {
//    vec3 ambient;
//    vec3 diffuse;
//    vec3 specular;
//    float shininess;
//};

// uniform vec3 lightPos;
// uniform vec3 viewPos;

layout (std140, binding = 4) uniform Material {
    uniform vec4 texBaseColorFactor;
};

uniform sampler2D myTexture;

out vec4 FragColor;

void main() {
    vec4 texColor = texture(myTexture, vsOut.texCoords) * texBaseColorFactor;
    FragColor = texColor;
}

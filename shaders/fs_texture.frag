#version 460 core

in VsOut {
    vec2 texCoords;
    //vec3 normal;
    //vec3 fragPos;
} vsOut;

out vec4 FragColor;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

// uniform vec3 lightPos;
// uniform vec3 viewPos;

uniform Material material;
uniform vec4 texBaseColorFactor;
uniform sampler2D myTexture;

uniform uint drawingPoints;

void main() {
    if (drawingPoints == 1) {
        //vec4 texColor = texture(myTexture, texCoords) * texBaseColorFactor;
        //FragColor = vec4(texColor.xyz, texColor.w * globalAlpha);
        FragColor = texBaseColorFactor;
    } else {
        vec4 texColor = texture(myTexture, vsOut.texCoords) * texBaseColorFactor;
        FragColor = vec4(texColor.xyz, 1.0);
        //FragColor = vec4(0.4, 0.5, 0.9, 0.6);
    }
}

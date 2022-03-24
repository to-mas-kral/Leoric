#version 460 core
out vec4 FragColor;

in vec2 texCoords;
in vec3 normal;
in vec3 fragPos;

uniform vec3 lightPos;
uniform vec3 viewPos;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

uniform Material material;

uniform sampler2D myTexture;
uniform vec4 texBaseColorFactor;
uniform uint drawingPoints;

uniform float globalAlpha;

void main() {
    if (drawingPoints == 1) {
        //vec4 texColor = texture(myTexture, texCoords) * texBaseColorFactor;
        //FragColor = vec4(texColor.xyz, texColor.w * globalAlpha);
        FragColor = texBaseColorFactor;
    } else {
        vec4 texColor = texture(myTexture, texCoords) * texBaseColorFactor;
        FragColor = vec4(texColor.xyz, texColor.w * globalAlpha  * 0.5);
        FragColor = vec4(texColor.xyz, 1.0);
        //FragColor = vec4(texBaseColorFactor.xyz, texBaseColorFactor.w * globalAlpha * 0.7);
        //discard;
    }
}

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

void main() {
    vec4 texColor = texture(myTexture, texCoords);
    FragColor = texColor;

    /*// ambient
    vec4 ambientColor = vec4(material.ambient, 1.0) * texColor * 0.2;

    vec3 lightDir = normalize(lightPos - fragPos);
    vec3 norm = normalize(normal);

    // diffuse
    float diffuseK = max(dot(norm, lightDir), 0);
    vec4 diffuseColor = texColor * vec4(diffuseK * material.diffuse, 1.0);

    // specular
    vec3 viewDir = normalize(viewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec4 specular = texColor * vec4(material.specular * spec, 1.0);

    FragColor = ambientColor + diffuseColor + specular; */
}

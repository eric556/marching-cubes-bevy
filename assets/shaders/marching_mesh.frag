#version 450

layout(location = 0) out vec4 o_Target;

layout(location = 0) in vec3 normal;
layout(location = 1) in vec3 frag_pos;
layout(location = 2) out float data;


layout(set = 2, binding = 0) uniform MarchMeshMaterial_lightColor {
    vec3 lightColor;
};

layout(set = 2, binding = 1) uniform MarchMeshMaterial_objectColor {
    vec3 objectColor;
};

layout(set = 2, binding = 3) uniform MarchMeshMaterial_lightPos {
    vec3 lightPos;
};

void main() {
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
  	
    // diffuse 
    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(lightPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
            
    vec3 result = (ambient + diffuse) * normalize(frag_pos);
    o_Target = vec4(result, 1.0);
}

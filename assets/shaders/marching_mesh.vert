#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in float Vertex_Data;

layout(location = 0) out vec3 normal;
layout(location = 1) out vec3 frag_pos;
layout(location = 2) out float data;


layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    data = Vertex_Data;
    normal = mat3(transpose(inverse(Model))) * Vertex_Normal;  
    frag_pos = vec3(Model * vec4(Vertex_Position, 1.0));
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
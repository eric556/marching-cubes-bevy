#version 450

layout(location = 1) in vec3 v_Position;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform DebugCornerMaterial_value {
    float value;
};

void main() {
    o_Target = vec4(value, v_Position.y, v_Position.z, 1.0);
}
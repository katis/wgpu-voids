#version 450

layout(set = 0, binding = 1) uniform Locals {
    mat3 normalView;
};

layout(location = 0) out vec4 color;

void main() {
    color = vec4(0.3, 0.5, 0.1, 0.0);
}

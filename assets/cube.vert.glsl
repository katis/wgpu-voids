#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_Transform;
};

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec3 fragVert;

void main() {
    gl_Position = u_Transform * vec4(position, 1.0);
    // convert from -1,1 Z to 0,1
    gl_Position.z = 0.5 * (gl_Position.z + gl_Position.w);

    fragNormal = normal;
    fragVert = position;
}

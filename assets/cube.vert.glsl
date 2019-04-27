#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoord;
layout(location = 3) in mat4 model;
// layout(location = 4, 5, 6) in use by model

layout(set = 0, binding = 0) uniform Locals {
    mat4 projection_view;
};

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out vec3 fragVert;
layout(location = 2) out vec2 fragTexCoord;

void main() {
    mat4 mvp = projection_view * model;
    gl_Position = mvp * vec4(position, 1.0);
    // convert from -1,1 Z to 0,1

    fragTexCoord = texCoord;
    fragNormal = normal;
    fragVert = position;
}

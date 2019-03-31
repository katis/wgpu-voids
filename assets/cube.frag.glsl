#version 450

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec3 fragVert;
layout(location = 2) in vec2 fragTexCoord;

layout(set = 0, binding = 1) uniform Locals {
    mat4 view;
    mat3 normalView;
};

layout(set = 0, binding = 2) uniform Light {
    vec3 position;
    vec3 intensities; //a.k.a the color of the light
} light;

layout(set = 0, binding = 3) uniform texture2D textureColor;
layout(set = 0, binding = 4) uniform sampler samplerColor;

layout(location = 0) out vec4 color;

void main() {
    vec3 normal = normalize(normalView * fragNormal);
    vec3 fragPosition = vec3(view * vec4(fragVert, 1));
    vec3 surfaceToLight = light.position - fragPosition;
    float brightness = clamp(dot(normal, surfaceToLight) / (length(surfaceToLight) * length(normal)), 0, 1);
    vec4 surfaceColor = texture(sampler2D(textureColor, samplerColor), fragTexCoord);
    color = vec4(brightness * light.intensities * surfaceColor.rgb, surfaceColor.a);
}

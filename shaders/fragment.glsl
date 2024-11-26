#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) flat in int InstanceIndex;

layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
    vec4 dir;
} camera;


void main() {
    int index = InstanceIndex;

    outColor = vec4(fragColor, 1.0);
}

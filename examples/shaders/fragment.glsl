#version 450

layout(location = 0) out vec4 f_color;

layout(location = 0) in vec4 vertex_pos;
layout(location = 1) in vec4 vertex_norm;

layout(set = 0, binding = 0) uniform CameraUniform
{
    mat4 camera_matrix;
    vec4 cam_pos;
};


void main() {

    vec3 light_dir = normalize(vec3(0.0, -1.0, -1.0));

    float light_val = dot(vertex_norm.xyz, -light_dir) / 2.0 + 0.5;

    f_color = vec4(vec3(light_val), 1.0);
}

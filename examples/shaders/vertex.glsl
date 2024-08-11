#version 450

layout (location = 0) in vec4 vertex_pos;
layout (location = 1) in vec4 vertex_norm;

layout (location = 0) out vec4 pos;
layout (location = 1) out vec4 normal;

layout (set = 0, binding = 0) uniform CameraUniform {
    mat4 camera_matrix;
    vec4 cam_pos;
};

void main() {
    gl_Position = camera_matrix * vertex_pos;
    pos = vertex_pos;
    normal = vertex_norm;
}

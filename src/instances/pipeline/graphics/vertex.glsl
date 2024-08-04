#version 450

layout (location = 0) in vec4 pos;
layout (location = 0) out vec4 pos_out;
layout (location = 1) out flat uint InstanceIndex;

layout (set = 0, binding = 0) buffer Data {
    mat4 model_matrix[];
};

layout (set = 0, binding = 1) buffer CameraUniform {
    mat4 camera_matrix;
    vec4 cam_pos;
};

void main() {
    gl_Position = camera_matrix * model_matrix[gl_InstanceIndex] * pos;
    InstanceIndex = gl_InstanceIndex;
    pos_out = pos;
}

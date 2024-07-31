#version 450

layout (location = 0) in vec4 pos;
layout (location = 0) out vec4 pos_out;

layout (set = 0, binding = 0) buffer Data {
    vec4 model_pos[];
};

void main() {
    gl_Position = model_pos[gl_InstanceIndex] + pos;
    pos_out = pos;
}

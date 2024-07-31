#version 450

layout (location = 0) out vec4 f_color;
layout (location = 0) in vec4 pos_out;

void main() {
    f_color = pos_out;
}

#version 450

layout(local_size_x=4, local_size_y=1, local_size_z=1) in;


layout(set=0, binding=0) buffer Data {
    int data[];
} buff;


void main() {
    uint x = gl_GlobalInvocationID.x;

    buff.data[x] = 300;
}







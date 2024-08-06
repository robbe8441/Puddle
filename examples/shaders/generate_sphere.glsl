#version 450

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(set = 0, binding = 3, r8ui) uniform uimage3D image;

void main() {
    uvec3 pos = gl_GlobalInvocationID;
    vec3 position = vec3(pos);

    vec3 center = vec3(15.0);
    float radius = 10.0;

    float distance = length(position - center);

    if (distance < radius) {
        imageStore(image, ivec3(pos), uvec4(255, 0, 0, 0)); 
    } else {
        imageStore(image, ivec3(pos), uvec4(0, 0, 0, 0)); 
    }
}


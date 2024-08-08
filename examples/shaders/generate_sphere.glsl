#version 450

layout(set = 0, binding = 2) buffer CameraUniform {
    mat4 camera_matrix;
    vec4 cam_pos;
};

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(set = 0, binding = 3, r8ui) uniform uimage3D image;

float noiseFn(vec3 pos) {
    return length(sin(pos.xy / 20.0) + cos(cam_pos.xz)) / 4.0 + 1.0;
}

void main() {
    uvec3 pos = gl_GlobalInvocationID;
    vec3 position = vec3(pos);

    vec3 center = vec3(150.0);
    float radius = 100.0;

    float distance = length(position - center) * noiseFn(position);

    if (distance < radius) {
        imageStore(image, ivec3(pos), uvec4(255, 0, 0, 0));
    } else {
        imageStore(image, ivec3(pos), uvec4(0, 0, 0, 0));
    }
}

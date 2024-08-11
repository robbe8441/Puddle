#version 450

layout(location = 0) out vec4 f_color;
layout(location = 0) in vec4 pos_out;
layout(location = 1) in flat uint InstanceIndex;

layout(set = 0, binding = 1) buffer Data {
    mat4 model_matrices[];
};

layout(set = 0, binding = 2) buffer CameraUniform {
    mat4 camera_matrix;
    vec4 cam_pos;
};

layout(set = 0, binding = 3, r8ui) uniform uimage3D image;


void main() {
    mat4 model_matrix = model_matrices[InstanceIndex];

    vec3 cam_pos_model = vec3(model_matrix * vec4(cam_pos.xyz, 1.0));

    vec3 ray_dir = normalize(pos_out.xyz - cam_pos_model);

    for (int i = 0; i < 400; i++) {
        vec3 pos = (pos_out.xyz + ray_dir * (i/100.0)) + 1.0;

        uint v = uint(imageLoad(image, ivec3(pos * 150.0)));

        if (v != 0) {
            f_color = vec4(vec3(i / 300.0), 1.0);
            return;
        }
    }

    f_color = vec4(0.0);
}

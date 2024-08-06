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


float sdBoxFrame(vec3 p, vec3 b, float e)
{
    p = abs(p) - b;
    vec3 q = abs(p + e) - e;
    return min(min(
            length(max(vec3(p.x, q.y, q.z), 0.0)) + min(max(p.x, max(q.y, q.z)), 0.0),
            length(max(vec3(q.x, p.y, q.z), 0.0)) + min(max(q.x, max(p.y, q.z)), 0.0)),
        length(max(vec3(q.x, q.y, p.z), 0.0)) + min(max(q.x, max(q.y, p.z)), 0.0));
}

void main() {
    // mat4 model_matrix = model_matrices[InstanceIndex];
    //
    // vec3 cam_pos_model = vec3(model_matrix * vec4(cam_pos.xyz, 1.0));
    //
    // vec3 ray_dir = normalize(pos_out.xyz - cam_pos_model);
    //
    // float h = 0;
    // for (int i = 0; i < 400; i++) {
    //     vec3 pos = mod((pos_out.xyz + ray_dir * h), vec3(1.0)) - 0.5;
    //
    //     float box = sdBoxFrame(pos, vec3(0.2), 0.01);
    //
    //     if (box < 0.01) {
    //         f_color = vec4(vec3(i / 300.0), 1.0);
    //         return;
    //     }
    //     h += box;
    // }

    ivec3 pos = ivec3(pos_out * 30.0) + ivec3(0,0,15);
    float v = float(imageLoad(image, pos)) / 255.0;

    f_color = vec4(vec3(v), 1.0);
}

#version 450

layout(location = 0) out vec4 f_color;
layout(location = 0) in vec4 pos_out;
layout(location = 1) in flat uint InstanceIndex;

layout(set = 0, binding = 1) buffer CameraUniform {
    mat4 camera_matrix;
    vec4 cam_pos;
};

layout(set = 0, binding = 0) buffer Data {
    mat4 model_matrices[];
};

float sdBoxFrame(vec3 p, vec3 b, float e)
{
    p = abs(p) - b;
    vec3 q = abs(p + e) - e;
    return min(min(
            length(max(vec3(p.x, q.y, q.z), 0.0)) + min(max(p.x, max(q.y, q.z)), 0.0),
            length(max(vec3(q.x, p.y, q.z), 0.0)) + min(max(q.x, max(p.y, q.z)), 0.0)),
        length(max(vec3(q.x, q.y, p.z), 0.0)) + min(max(q.x, max(q.y, p.z)), 0.0));
}

float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float sdPyramid(vec3 p, float h)
{
    float m2 = h * h + 0.25;

    p.xz = abs(p.xz);
    p.xz = (p.z > p.x) ? p.zx : p.xz;
    p.xz -= 0.5;

    vec3 q = vec3(p.z, h * p.y - 0.5 * p.x, h * p.x + 0.5 * p.y);

    float s = max(-q.x, 0.0);
    float t = clamp((q.y - 0.5 * p.z) / (m2 + 0.25), 0.0, 1.0);

    float a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;
    float b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) + (q.y - m2 * t) * (q.y - m2 * t);

    float d2 = min(q.y, -q.x * m2 - q.y * 0.5) > 0.0 ? 0.0 : min(a, b);

    return sqrt((d2 + q.z * q.z) / m2) * sign(max(q.z, -p.y));
}

void main() {
    mat4 model_matrix = model_matrices[InstanceIndex];

    vec3 cam_pos_model = vec3(inverse(model_matrix) * vec4(cam_pos.xyz, 1.0));

    vec3 ray_dir = normalize(pos_out.xyz - cam_pos_model);

    for (int i = 0; i < 400; i++) {
        vec3 pos = pos_out.xyz + ray_dir * (i / 50.0);
        float box = sdBoxFrame(pos, vec3(1.0), 0.05);
        float sphere = sdPyramid(pos + vec3(0.0, 0.5, 0.0), 1.0);

        if (min(box, sphere) < 0.01) {
            if (sphere < box) {
                f_color = vec4(vec3(i / 10.0, 0.8, 0.5), 1.0);
            } else {
                f_color = vec4(vec3(i / 300.0), 1.0);
            }
            return;
        }
    }

    f_color = vec4(0.0);
}


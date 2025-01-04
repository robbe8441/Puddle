import octree;
import bindless;

struct AssembledVertex {
  [[vk::location(0)]]
  float4 position;
  uint index : SV_VertexID;
};

struct VertexStageOutput {
  float4 sv_position : SV_Position;
  float3 vertex_pos;
  float3 color;
};

struct Uniforms {
  float4x4 camera;
  float4 cam_pos;
  float time;
};

[shader("vertex")]
VertexStageOutput vs_main(AssembledVertex assembledVertex) {
  VertexStageOutput output;

  float3 colors[] = {
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0),
  };

  let uniform = GetUniformBuffer<Uniforms>(0);

  output.sv_position = mul(uniform.camera, assembledVertex.position);
  output.color = colors[assembledVertex.index % 3];
  output.vertex_pos = assembledVertex.position.xyz;

  return output;
}

struct FragmentOutput {
  float4 color : SV_Target;
  float4 normal;
  float depth;
};

[shader("fragment")]
FragmentOutput fs_main(VertexStageOutput input) {
  let uniform = GetUniformBuffer<Uniforms>(0);

  var hit : Hit;
  let ray_dir = normalize(input.vertex_pos - uniform.cam_pos.xyz);
  let ray = Ray(uniform.cam_pos.xyz, ray_dir, 1.0f / ray_dir);

  let color_index = trace_ray(0, ray, hit);

  FragmentOutput output;
  output.color = float4(float3(color_index / 255.0), 1.0);
  output.normal = 1.0 - output.color;
  output.depth = 1.0 - output.color.r;

  return output;
}



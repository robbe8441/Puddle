
struct AssembledVertex {
  [[vk::location(0)]]
  float4 position;
  uint index : SV_VertexID;
};

struct VertexStageOutput {
  float4 sv_position : SV_Position;
  float3 color;
};

struct Uniforms {
  float4x4 camera;
  float data;
};

[[vk::binding(0)]]
ConstantBuffer<Uniforms> uniform_buffers[];

[shader("vertex")]
VertexStageOutput vs_trig(AssembledVertex assembledVertex) {
  VertexStageOutput output;

  float3 colors[] = {
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0),
  };

  output.sv_position = mul(uniform_buffers[0].camera, assembledVertex.position);
  // output.sv_position = assembledVertex.position;
  output.color = colors[assembledVertex.index % 3];

  output.color.r = uniform_buffers[0].data;

  return output;
}

// Fragment Shader
struct FragmentOutput {
  float4 color : SV_Target;
  float depth;
};

[shader("fragment")]
FragmentOutput main_fs(VertexStageOutput input) {
  FragmentOutput output;
  output.color = float4(input.color, 1.0);
  output.depth = uniform_buffers[0].data;

  return output;
}

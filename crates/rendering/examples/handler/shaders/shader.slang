struct AssembledVertex {
  [[vk::location(0)]] float4 position;
  uint index : SV_VertexID;
};

struct VertexStageOutput {
  float4 sv_position : SV_Position;
  float3 color;
};

[shader("vertex")]
VertexStageOutput vs_trig(AssembledVertex assembledVertex) {
  VertexStageOutput output;

  float3 colors[] = {
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0),
  };

  output.sv_position = assembledVertex.position;
  output.color = colors[assembledVertex.index % 3];

  return output;
}

// Fragment Shader

[shader("fragment")]
float4 main_fs(VertexStageOutput input) : SV_Target {
  return float4(input.color, 1.0);
}

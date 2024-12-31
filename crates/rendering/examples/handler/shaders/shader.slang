struct AssembledVertex {
  uint index : SV_VertexID;
};

struct VertexStageOutput {
  float4 sv_position : SV_Position;
  float3 color;
};

[shader("vertex")]
VertexStageOutput vs_trig(AssembledVertex assembledVertex) {
  VertexStageOutput output;

  float3 positions[] = {
    float3(-0.5, 0.5, 1.0),
    float3(0.0, -0.5, 1.0),
    float3(0.5, 0.5, 1.0),
  };

  float3 colors[] = {
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0),
  };

  output.sv_position = float4(positions[assembledVertex.index], 1.0);
  output.color = colors[assembledVertex.index];

  return output;
}

// Fragment Shader

[shader("fragment")]
float4 main_fs(VertexStageOutput input) : SV_Target {
  return float4(input.color, 1.0);
}

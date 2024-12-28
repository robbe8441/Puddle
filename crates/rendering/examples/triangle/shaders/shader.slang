
struct AssembledVertex {
  uint index : SV_VertexID;
};

struct VertexStageOutput {
  float4 sv_position : SV_Position;
};

[shader("vertex")]
VertexStageOutput vertexMain(AssembledVertex assembledVertex) {
  VertexStageOutput output;

  float3 positions[] = {
    float3(0.5, 0.5, 0.5),
    float3(0.5, 0.5, 0.5),
    float3(0.5, 0.5, 0.5),
  };

  output.sv_position = float4(positions[assembledVertex.index], 1.0);

  return output;
}

// Fragment Shader

[shader("fragment")]
float4 main_fs(VertexStageOutput input) : SV_Target {
  float3 color = input.sv_position.xyz;

  return float4(color, 1.0);
}

import types;

[shader("vertex")]
VertexStageOutput vs_quad(AssembledVertex assembledVertex) {
  VertexStageOutput output;

  float3 positions[] = {
    float3(-0.5, 0.5, 1.0), float3(-0.5, -0.5, 1.0), float3(0.5, -0.5, 1.0),

    float3(-0.5, 0.5, 1.0), float3(0.5, -0.5, 1.0),  float3(0.5, 0.5, 1.0),
  };

  float3 colors[] = {
    float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0), float3(0.0, 0.0, 1.0),

    float3(1.0, 0.0, 0.0), float3(0.0, 0.0, 1.0), float3(1.0, 1.0, 0.0),
  };

  output.sv_position = float4(positions[assembledVertex.index], 1.0);
  output.color = colors[assembledVertex.index];

  return output;
}

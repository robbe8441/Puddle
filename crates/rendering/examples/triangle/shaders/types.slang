
struct AssembledVertex {
  uint index : SV_VertexID;
};

struct VertexStageOutput {
  float4 sv_position : SV_Position;
  float3 color;
};

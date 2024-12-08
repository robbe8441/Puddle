struct Camera {
  float4x4 view_proj;
  float4 cam_pos;
};

struct Instance {
  float4x4 transform;
};


struct InstanceArray {
  Instance data[];
}

[[vk::push_constant]]
cbuffer PushConstants {
  uint camera;
  uint swapchain_image;
  uint insance_array;
  uint instance_count;
};

[[vk::binding(0)]]
ConstantBuffer<Camera> camera_buffers[];

[[vk::binding(1)]]
StructuredBuffer<InstanceArray> instance_data[];

[[vk::binding(3)]]
RWTexture2D<float4> swapchain_images[];

[shader("compute")]
[numthreads(32, 32, 1)]
func main(uint3 threadId: SV_DispatchThreadID) {
  let screen = swapchain_images[swapchain_image];

  uint width, height, levels = 0;
  screen.GetDimensions(0, width, height, levels);

  float2 tex_coords = float2(threadId.xy) / float2(width, height);

  screen.Store(threadId.xy, float4(float3(swapchain_image / 5.0), 1.0));
}


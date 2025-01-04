
[[vk::binding(0)]]
__DynamicResource<__DynamicResourceKind.General> g_uniform_heap[];

[[vk::binding(1)]]
__DynamicResource<__DynamicResourceKind.General> g_storage_heap[];

[[vk::binding(2)]]
__DynamicResource<__DynamicResourceKind.General> g_storeage_image_heap[];

ConstantBuffer<T> GetUniformBuffer<T>(uint index) {
  return g_uniform_heap[index].as<ConstantBuffer<T>>();
}

StructuredBuffer<T> GetStorageBuffer<T>(uint index) {
  return g_storage_heap[index].as<StructuredBuffer<T>>();
}

import bindless;

struct Ray {
  float3 o, d, invDir;
};
struct Hit {
  float3 p;
  float t;
  float tmax;
  float tmin;
  float3 n;
};

static float3 NODE_POS[8] = {
    float3(-1.0, -1.0, -1.0),
    float3(1.0, -1.0, -1.0),
    float3(-1.0, 1.0, -1.0),
    float3(1.0, 1.0, -1.0),
    float3(-1.0, -1.0, 1.0),
    float3(1.0, -1.0, 1.0),
    float3(-1.0, 1.0, 1.0),
    float3(1.0, 1.0, 1.0),
};

bool BBoxIntersect(const float3 boxMin, const float3 boxMax, const Ray r,
                   out Hit hit) {
  float3 tbot = r.invDir * (boxMin - r.o);
  float3 ttop = r.invDir * (boxMax - r.o);
  float3 tmin = min(ttop, tbot);
  float3 tmax = max(ttop, tbot);
  float2 t = max(tmin.xx, tmin.yz);
  float t0 = max(t.x, t.y);
  t = min(tmax.xx, tmax.yz);
  float t1 = min(t.x, t.y);
  hit.tmin = t0;
  hit.tmax = t1;
  return t1 > max(t0, 0.0);
}

float3 box_normal(float3 center, float3 box_min, float3 box_max, float3 hit) {
  float3 p = hit - center;
  float3 d = (box_min - box_max) * 0.5;
  float bias = 1.00001;

  return normalize(float3(
      	trunc(p.x / abs(d.x) * bias),
        trunc(p.y / abs(d.y) * bias),
        trunc(p.z / abs(d.z) * bias)));
}

struct VoxelData {
  uint64_t colors; // TODO
  uint child_desc;
  uint _padding;

  uint get_color(uint index) {
    return (uint)(this.colors >> (index * 8)) & 0xFF;
  }

  uint get_valid_mask() {
    return (this.child_desc >> 24) & 0xFF;
  }

  uint get_child_ptr() {
    return (this.child_desc & 0xFFFFFF);
  }

};


uint trace_ray(uint octree_index, Ray ray, out Hit hit) {
    let voxel_data = GetStorageBuffer<VoxelData>(octree_index);

    uint index = 0;
    float scale = 50.0;
    float3 center = float3(0.0);
    float3 minBox = center - scale;
    float3 maxBox = center + scale;

    struct Stack {
        uint index;
        float3 center;
        float scale;
    };
    Stack stack[20];
    uint stack_index = 0;

    scale *= 0.5;

    stack[stack_index++] = Stack(0u, center, scale);

    var stepps = 0;

    while(stack_index-- > 0) {
        center = stack[stack_index].center;
        index = stack[stack_index].index;
        scale = stack[stack_index].scale;

        let voxel_node = voxel_data[index];

        uint voxel_child_ptr = voxel_node.get_child_ptr();

        uint voxel_valid_mask = voxel_node.get_valid_mask();

        uint child_ptr_offset = 0u;
        stepps++;

        for (uint i = 0u; i < 8u; ++i) {
            bool empty = voxel_node.get_color(i) == 0;
            bool is_leaf = voxel_valid_mask << i == 0;
            if (empty){
                continue;
            }

            
            float3 new_center = center + scale * NODE_POS[i];
            float3 minBox = new_center - scale;
            float3 maxBox = new_center + scale;
            
            if (!BBoxIntersect(minBox, maxBox, ray, hit)){
                if(!is_leaf){
                   child_ptr_offset += 1u;
                }
                continue;
            }


            if (is_leaf){
                let hit_pos = ray.o + ray.d * hit.tmin;
                hit.n = box_normal(new_center, minBox, maxBox, hit_pos);
                return voxel_node.get_color(i);
            } else {
                stack[stack_index++] = Stack(voxel_child_ptr+child_ptr_offset, new_center, scale*0.5f);
                child_ptr_offset += 1u;
            }
        }
    }

    return 0;
}

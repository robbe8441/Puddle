
# glslc -fshader-stage=vertex ./shaders/vertex.glsl -o ./shaders/vertex.spv 
# glslc -fshader-stage=fragment ./shaders/fragment.glsl -o ./shaders/fragment.spv 
# glslc -fshader-stage=compute ./shaders/compute.glsl -o ./shaders/compute.spv 

slang=~/Programs/slang/bin/slangc

$slang ./shaders/shader.slang -target spirv -o ./shaders/shader.spv

# dxc -T vs_6_0 -spirv ./shaders/vertex.hlsl -Fo ./shaders/vertex_hlsl.spv
# dxc -T ps_6_0 -spirv ./shaders/fragment.hlsl -Fo ./shaders/fragment_hlsl.spv


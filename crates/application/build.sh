slang=~/Programs/slang/bin/slangc

# Compile
$slang -O3 ./shaders/shader.slang -target spirv -o ./shaders/shader.spv
spirv-opt -o ./shaders/shader.spv ./shaders/shader.spv


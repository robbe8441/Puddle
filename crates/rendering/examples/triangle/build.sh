slang=~/Programs/slang/bin/slangc

# Compile
$slang ./shaders/shader.slang -target spirv -o ./shaders/shader.spv

# Optimize
spirv-opt ./shaders/shader.spv -o ./shaders/shader_opt.spv

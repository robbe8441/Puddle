slang=~/Programs/slang/bin/slangc

# Compile
$slang ./shaders/shader.slang -target spirv -o ./shaders/shader.spv
$slang ./shaders/quad.slang -target spirv -o ./shaders/quad.spv

# $slang --help

# Optimize
spirv-opt ./shaders/shader.spv -o ./shaders/shader_opt.spv

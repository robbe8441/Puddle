slang=~/Programs/slang/bin/slangc

# Compile
$slang ./shaders/shader.slang -target spirv -o ./shaders/shader.spv

# $slang --help

# Optimize
spirv-opt ./shaders/shader.spv -o ./shaders/shader.spv

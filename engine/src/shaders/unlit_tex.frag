#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 fragColor;

/*
layout(binding = 1) uniform Animation {
    float anim;
};
*/

layout(location = 0) out vec4 outColor;
layout(binding = 1) uniform sampler2D tex;

void main() {
    outColor = vec4(texture(tex, fragColor.xy).rgb, 1.0);
}

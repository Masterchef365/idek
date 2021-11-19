#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

// Per-frame UBO
layout(binding = 0) uniform PerFrame {
    mat4 camera[2];
    float u_time;
};

// Returns true where `x` is within `width` of an integer 
bool integer_lines(float x, const float width) {
    return abs(fract(x + .5) - .5) < width;
}

const vec2 u_resolution = vec2(600);

void main() {
    vec2 st = (gl_FragCoord.xy/u_resolution.xy) * 2. - 1.;
    
    const float scale = 2.;
    vec2 frac = fract(st * vec2(1., 2.) * scale);

    vec3 color = vec3(mix(
        frac.y, 
        1. - frac.y, 
        float(frac.x > .5)
    ));

    outColor = vec4(color * fragColor, 1.);
}
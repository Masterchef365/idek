#version 450
#extension GL_EXT_multiview : require

// Per-frame UBO
layout(binding = 0) uniform PerFrame {
    mat4 camera[2];
    float anim;
};

// Model matrices
layout(binding = 1) buffer Models {
    mat4 model_mats[];
};

// Resource indices
layout(push_constant) uniform Indices {
    uint model_index;
};

// Vertex data
layout(location = 0) in vec3 vert_pos;
layout(location = 1) in vec3 vert_color;

// Fragment outputs
layout(location = 0) out vec3 frag_color;

void main() {
    gl_Position = camera[gl_ViewIndex]
        * model_mats[model_index]
        * vec4(vert_pos, 1.0);
    frag_color = vert_color;
}


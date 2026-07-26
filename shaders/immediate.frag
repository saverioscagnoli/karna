#version 450

// SDL GPU SPIR-V convention: fragment samplers live in set 2.
layout(set = 2, binding = 0) uniform sampler2D atlas_page;

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_uv_coords;

layout(location = 0) out vec4 frag_color;

void main() {
    // Untextured geometry samples the atlas' reserved white texel, so solid
    // shapes and images share one pipeline.
    frag_color = v_color * texture(atlas_page, v_uv_coords);
}

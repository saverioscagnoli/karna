#version 450

// SDL GPU SPIR-V convention: fragment combined image samplers live in set 2.
layout(set = 2, binding = 0) uniform sampler2D imgui_texture;

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_uv_coords;

layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = v_color * texture(imgui_texture, v_uv_coords);
}

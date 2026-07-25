#version 450

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_uv_coords;

layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = v_color;
}

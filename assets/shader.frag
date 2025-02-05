#version 460 core

in vec4 vertex_color;
in vec2 tex_coords;

out vec4 color;

void main() {
    color = vertex_color;
}
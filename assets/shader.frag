#version 460 core

in vec4 vertex_color;
in vec2 tex_coords;

out vec4 color;

uniform sampler2D tex;

void main() {
    vec4 texture_color = texture(tex, tex_coords);
    color = texture_color * vertex_color;
}
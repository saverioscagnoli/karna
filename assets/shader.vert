#version 460 core

layout(location = 0) in vec3 aPosition;
layout(location = 1) in vec4 aColor;
layout(location = 2) in vec2 aTexCoords;

out vec4 vertex_color;
out vec2 tex_coords;

uniform mat4 uProjection;

void main() {
    gl_Position = uProjection * vec4(aPosition, 1.0);
    vertex_color = aColor;
    tex_coords = aTexCoords;

}


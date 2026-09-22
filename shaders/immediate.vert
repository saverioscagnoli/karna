#version 450

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec4 a_color;
layout(location = 2) in vec2 a_uv;
layout(location = 3) in float a_page;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec3 v_uv;

layout(set = 1, binding = 0) uniform Camera {
    mat4 mvp;
};

void main() {
    gl_Position = mvp * vec4(a_position, 1.0);
    v_color = a_color;
    v_uv = vec3(a_uv, a_page);
}

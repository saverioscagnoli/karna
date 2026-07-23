#version 450

layout(set = 1, binding = 0) uniform Camera {
    mat4 view_projection;
};

layout(set = 1, binding = 1) uniform Model {
    mat4 model;
    mat4 normal_matrix;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;

void main() {
    gl_Position = view_projection * model * vec4(position, 1.0);
    v_normal = mat3(normal_matrix) * normal;
    v_uv = uv;
}

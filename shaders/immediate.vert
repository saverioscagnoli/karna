#version 450

// SDL GPU SPIR-V convention: vertex uniform buffers live in set 1.
layout(set = 1, binding = 0) uniform Camera {
    mat4 view_projection;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 uv_coords;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_uv_coords;

void main() {
    // Apply camera view-projection directly (no model transform)
    gl_Position = view_projection * vec4(position, 1.0);
    v_color = color;
    v_uv_coords = uv_coords;
}

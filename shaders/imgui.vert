#version 450

// SDL GPU SPIR-V convention: vertex uniform buffers live in set 1.
layout(set = 1, binding = 0) uniform Projection {
    mat4 mvp;
};

// Matches ImDrawVert: vec2 pos, vec2 uv, packed RGBA8 colour.
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv_coords;
layout(location = 2) in vec4 color;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_uv_coords;

void main() {
    gl_Position = mvp * vec4(position, 0.0, 1.0);
    v_color = color;
    v_uv_coords = uv_coords;
}

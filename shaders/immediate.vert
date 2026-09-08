#version 450

// The immediate batcher's only vertex format. Every primitive -- rects today,
// images and glyphs tomorrow -- is quads in this layout, so one pipeline draws
// the whole frame and a texture change is the only thing that splits a batch.

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec4 a_color;
layout(location = 2) in vec2 a_uv;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_uv;

// set 1 is where SDL's GPU layer expects vertex uniform buffers to live.
layout(set = 1, binding = 0) uniform Camera {
    mat4 u_proj;
};

void main() {
    v_color = a_color;
    v_uv = a_uv;
    gl_Position = u_proj * vec4(a_pos, 0.0, 1.0);
}

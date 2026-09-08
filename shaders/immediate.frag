#version 450

// Untextured primitives sample a 1x1 white texel, so they take the same path as
// textured ones and the vertex color acts as a tint either way.

layout(location = 0) in vec4 v_color;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 o_color;

// set 2 is where SDL's GPU layer expects fragment samplers to live.
layout(set = 2, binding = 0) uniform sampler2D u_texture;

void main() {
    o_color = v_color * texture(u_texture, v_uv);
}

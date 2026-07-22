#version 450

// SDL GPU SPIR-V convention: vertex uniform buffers live in set 1.
layout(set = 1, binding = 0) uniform Transform {
    vec2 scale;
    vec2 translate;
};

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv_coords;
layout(location = 2) in vec4 color;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_uv_coords;

// Imgui vertex colors are sRGB bytes, but the swapchain is a linear
// (sRGB-view) target, so they must be linearized before blending.
vec3 srgb_to_linear(vec3 c) {
    vec3 lo = c / 12.92;
    vec3 hi = pow((c + 0.055) / 1.055, vec3(2.4));
    return mix(lo, hi, step(vec3(0.04045), c));
}

void main() {
    gl_Position = vec4(position * scale + translate, 0.0, 1.0);
    v_color = vec4(srgb_to_linear(color.rgb), color.a);
    v_uv_coords = uv_coords;
}

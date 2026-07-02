@vs vs
// Camera view-projection matrix (uniform block)
layout(binding=0) uniform vs_params {
    mat4 view_projection;
};

// Vertex inputs (per-vertex data only)
in vec3 position;   // @location(0)
in vec4 color;      // @location(1)

// Vertex -> fragment varying
out vec4 v_color;

void main() {
    // Apply camera view-projection directly (no model transform)
    gl_Position = view_projection * vec4(position, 1.0);
    v_color = color;
}
@end

@fs fs
in vec4 v_color;

out vec4 frag_color;

void main() {
    // For text: vertex color defines text color, texture provides alpha mask
    // For rects: white texture pixel, so vertex color passes through
    frag_color = v_color;
}
@end

@program shader vs fs

// Vertex shader input (per-vertex data)
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

// Instance data (per-instance data)
struct InstanceInput {
    @location(3) instance_position: vec3<f32>,
    @location(4) instance_scale: vec3<f32>,
    @location(5) instance_rotation: vec3<f32>,
    @location(6) instance_color: vec4<f32>,
}

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// Camera view-projection matrix
@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

// Helper function to create a 2D rotation matrix around Z axis
fn rotation_z(angle: f32) -> mat4x4<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat4x4<f32>(
        vec4<f32>(c, s, 0.0, 0.0),
        vec4<f32>(-s, c, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

// Helper function to create a scale matrix
fn scale_matrix(scale: vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(scale.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, scale.z, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

// Helper function to create a translation matrix
fn translation_matrix(translation: vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(translation.x, translation.y, translation.z, 1.0)
    );
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Build transformation matrix: Translation * Rotation * Scale
    let scale_mat = scale_matrix(instance.instance_scale);
    let rotation_mat = rotation_z(instance.instance_rotation.z);
    let translation_mat = translation_matrix(instance.instance_position);

    // Apply transformations in order: scale -> rotate -> translate
    let model_matrix = translation_mat * rotation_mat * scale_mat;

    // Transform vertex position
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);

    // Apply camera view-projection
    out.clip_position = view_projection * world_position;

    // Blend vertex color with instance color
    out.color = vertex.color * instance.instance_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

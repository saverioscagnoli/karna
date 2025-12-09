use wgpu::ShaderModule;

/// Creates and returns the default shader module for the renderer.
/// This shader handles vertex transformations, instancing, and fragment coloring.
pub fn create_default_shader(device: &wgpu::Device) -> ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Default Shader"),
        source: wgpu::ShaderSource::Wgsl(DEFAULT_SHADER.into()),
    })
}

/// Default WGSL shader source code.
///
/// This shader supports:
/// - Vertex positions and colors
/// - Instance-based rendering with position, scale, and rotation
/// - Camera view-projection matrix
/// - Per-instance color tinting
pub const DEFAULT_SHADER: &str = r#"
struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var texture_atlas: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct InstanceInput {
    @location(3) instance_position: vec3<f32>,
    @location(4) instance_scale: vec3<f32>,
    @location(5) instance_rotation: vec3<f32>,
    @location(6) instance_color: vec4<f32>,
    @location(7) uv_offset: vec2<f32>,
    @location(8) uv_scale: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

fn rotation_matrix_z(angle: f32) -> mat4x4<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat4x4<f32>(
        vec4<f32>(c, s, 0.0, 0.0),
        vec4<f32>(-s, c, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
}

fn rotation_matrix_y(angle: f32) -> mat4x4<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat4x4<f32>(
        vec4<f32>(c, 0.0, -s, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(s, 0.0, c, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
}

fn rotation_matrix_x(angle: f32) -> mat4x4<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, c, s, 0.0),
        vec4<f32>(0.0, -s, c, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Create transformation matrices
    let scale_matrix = mat4x4<f32>(
        vec4<f32>(instance.instance_scale.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, instance.instance_scale.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, instance.instance_scale.z, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );

    let rotation_x = rotation_matrix_x(instance.instance_rotation.x);
    let rotation_y = rotation_matrix_y(instance.instance_rotation.y);
    let rotation_z = rotation_matrix_z(instance.instance_rotation.z);
    let rotation_matrix = rotation_z * rotation_y * rotation_x;

    let translation_matrix = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(instance.instance_position.x, instance.instance_position.y, instance.instance_position.z, 1.0),
    );

    // Apply transformations: scale -> rotate -> translate
    let model_matrix = translation_matrix * rotation_matrix * scale_matrix;
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);

    out.clip_position = camera.view_proj * world_position;

    // Multiply vertex color by instance color for tinting
    out.color = vertex.color * instance.instance_color;

    // Apply UV transform for texture atlas region mapping
    out.uv = instance.uv_offset + vertex.uv * instance.uv_scale;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture if UV coordinates are valid (non-zero or within bounds)
    let tex_color = textureSample(texture_atlas, texture_sampler, in.uv);

    // Multiply texture color by vertex color for tinting
    // If no texture is used (UVs are 0), the color will be just the vertex color
    return tex_color * in.color;
}
"#;

/// Creates a shader module from custom WGSL source code.
pub fn create_shader_from_source(device: &wgpu::Device, label: &str, source: &str) -> ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

pub struct Shaders;

// Replace your Shaders::basic() function with this instancing-capable version
impl Shaders {
    pub fn basic() -> &'static str {
        r#"
@group(0) @binding(0) var<uniform> view_projection: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct InstanceInput {
    @location(2) translation: vec3<f32>,
    @location(3) rotation: vec3<f32>,
    @location(4) scale: vec3<f32>,
    @location(5) instance_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// Helper function to create rotation matrix from Euler angles
fn rotation_matrix(rotation: vec3<f32>) -> mat4x4<f32> {
    let cos_x = cos(rotation.x);
    let sin_x = sin(rotation.x);
    let cos_y = cos(rotation.y);
    let sin_y = sin(rotation.y);
    let cos_z = cos(rotation.z);
    let sin_z = sin(rotation.z);
    
    // Rotation matrix: Rz * Ry * Rx
    return mat4x4<f32>(
        vec4<f32>(cos_y * cos_z, -cos_y * sin_z, sin_y, 0.0),
        vec4<f32>(cos_x * sin_z + sin_x * sin_y * cos_z, cos_x * cos_z - sin_x * sin_y * sin_z, -sin_x * cos_y, 0.0),
        vec4<f32>(sin_x * sin_z - cos_x * sin_y * cos_z, sin_x * cos_z + cos_x * sin_y * sin_z, cos_x * cos_y, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

@vertex
fn vs_main(
    input: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Create transformation matrices
    let scale_matrix = mat4x4<f32>(
        vec4<f32>(instance.scale.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, instance.scale.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, instance.scale.z, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
    
    let rotation_mat = rotation_matrix(instance.rotation);
    
let translation_matrix = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(
        instance.translation.x + instance.scale.x / 2.0, 
        instance.translation.y + instance.scale.y / 2.0, 
        instance.translation.z, 
        1.0
    )
);
    
    // Combine transformations: T * R * S
    let model_matrix = translation_matrix * rotation_mat * scale_matrix;

    var output: VertexOutput;
    output.position = view_projection * model_matrix * vec4<f32>(input.position, 1.0);
    output.color = input.color * instance.instance_color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
        "#
    }
}

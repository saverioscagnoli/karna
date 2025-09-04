pub struct Shaders;

impl Shaders {
    pub fn basic() -> &'static str {
        r#"
        @group(0) @binding(0) var<uniform> view_projection: mat4x4<f32>;
        
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) color: vec4<f32>,
        };

        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) color: vec4<f32>,
        };

        @vertex
        fn vs_main(input: VertexInput) -> VertexOutput {
            var output: VertexOutput;
            output.position = view_projection * vec4<f32>(input.position, 1.0);
            output.color = input.color;
            return output;
        }

        @fragment
        fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
            return input.color;
        }
        "#
    }
}

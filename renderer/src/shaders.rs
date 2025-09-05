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

    pub fn rect() -> &'static str {
        r#"
        @group(0) @binding(0) var<uniform> view_projection: mat4x4<f32>;
        
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) color: vec4<f32>,
            @location(2) instance_position: vec2<f32>,
            @location(3) instance_size: vec2<f32>,
            @location(4) instance_color: vec4<f32>,
        };

        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) color: vec4<f32>,
        };

        @vertex
        fn vs_main(input: VertexInput) -> VertexOutput {
            var output: VertexOutput;
            
            // Scale the unit quad vertex by the instance size
            let scaled_pos = input.position.xy * input.instance_size;
            
            // Translate by the instance position
            let final_pos = scaled_pos + input.instance_position;
            
            // Transform to clip space
            output.position = view_projection * vec4<f32>(final_pos, input.position.z, 1.0);
            
            // Use the instance color
            output.color = input.instance_color;
            
            return output;
        }

        @fragment
        fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
            return input.color;
        }
        "#
    }
}

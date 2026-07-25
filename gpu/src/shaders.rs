use logging::debug;
use sdl3::gpu::Shader;
use sdl3::gpu::ShaderFormat;
use sdl3::gpu::ShaderStage;
use utils::FastHashMap;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum ShaderRef {
    Builtin(usize),
}

pub struct ShaderDesc<'a> {
    pub vertex_spirv: &'a [u8],
    pub fragment_spirv: &'a [u8],
    pub vertex_uniform_buffers: u32,
    pub fragment_samplers: u32,
    pub fragment_uniform_buffers: u32,
}

pub struct ShaderPair {
    pub vertex: Shader,
    pub fragment: Shader,
}

#[derive(Default)]
pub struct ShaderRegistry {
    shaders: FastHashMap<usize, ShaderPair>,
}

impl ShaderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, device: &sdl3::gpu::Device, index: usize, desc: ShaderDesc) {
        let vertex = device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, desc.vertex_spirv, ShaderStage::Vertex)
            .with_uniform_buffers(desc.vertex_uniform_buffers)
            .with_entrypoint(c"main")
            .build()
            .expect("Failed to create vertex shader");

        let fragment = device
            .create_shader()
            .with_code(
                ShaderFormat::SPIRV,
                desc.fragment_spirv,
                ShaderStage::Fragment,
            )
            .with_samplers(desc.fragment_samplers)
            .with_uniform_buffers(desc.fragment_uniform_buffers)
            .with_entrypoint(c"main")
            .build()
            .expect("Failed to create fragment shader");

        self.shaders.insert(index, ShaderPair { vertex, fragment });
        debug!("Loaded builtin shader {}", index);
    }

    pub fn get(&self, r: &ShaderRef) -> &ShaderPair {
        match r {
            ShaderRef::Builtin(index) => self.shaders.get(index).expect("Failed to get shader"),
        }
    }
}

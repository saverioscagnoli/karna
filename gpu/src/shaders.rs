use std::borrow::Cow;

use logging::debug;
use utils::FastHashMap;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum ShaderRef {
    Builtin(usize),
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct ShaderRegistry {
    shaders: FastHashMap<usize, wgpu::ShaderModule>,
}

impl ShaderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<S>(&mut self, index: usize, src: S, device: &wgpu::Device)
    where
        S: Into<String>,
    {
        let src: String = src.into();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(src)),
        });

        self.shaders.insert(index, module);
        debug!("Loaded shader.");
    }

    pub fn get(&self, r: &ShaderRef) -> &wgpu::ShaderModule {
        match r {
            ShaderRef::Builtin(index) => self.shaders.get(index).expect("Failed to get shader"),
        }
    }
}

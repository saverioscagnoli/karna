use std::borrow::Cow;

use utils::FastHashMap;

use crate::GpuState;

#[derive(Debug)]
pub struct ShaderStore {
    shaders: FastHashMap<String, wgpu::ShaderModule>,
}

impl ShaderStore {
    pub(crate) fn new() -> Self {
        Self {
            shaders: FastHashMap::default(),
        }
    }

    pub fn load<L: AsRef<str>, S: Into<String>>(
        &mut self,
        label: L,
        src: S,
        device: &wgpu::Device,
    ) {
        let label = label.as_ref();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(src.into())),
        });

        self.shaders.insert(label.to_owned(), module);
    }

    pub fn get<L: AsRef<str>>(&self, label: L) -> &wgpu::ShaderModule {
        self.shaders
            .get(label.as_ref())
            .expect("Failed to get shader module")
    }
}

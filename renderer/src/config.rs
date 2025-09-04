use std::ops::{Deref, DerefMut};

pub struct RendererConfig {
    wgpu_config: wgpu::SurfaceConfiguration,
}

impl Deref for RendererConfig {
    type Target = wgpu::SurfaceConfiguration;

    fn deref(&self) -> &Self::Target {
        &self.wgpu_config
    }
}

impl DerefMut for RendererConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wgpu_config
    }
}

impl RendererConfig {
    pub fn new(wgpu_config: wgpu::SurfaceConfiguration) -> Self {
        Self { wgpu_config }
    }
}

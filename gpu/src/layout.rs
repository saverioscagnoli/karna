use std::num::NonZeroU32;

pub use wgpu::BindingType;
pub use wgpu::SamplerBindingType;
pub use wgpu::ShaderStages;
pub use wgpu::TextureSampleType;
pub use wgpu::TextureViewDimension;

use crate::GpuState;

pub struct LayoutBuilder {
    label: Option<&'static str>,
    visibility: wgpu::ShaderStages,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl LayoutBuilder {
    pub fn new(label: &'static str) -> Self {
        Self {
            label: Some(label),
            visibility: wgpu::ShaderStages::FRAGMENT,
            entries: Vec::new(),
        }
    }

    pub fn visibility(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.visibility = visibility;
        self
    }

    fn push(mut self, ty: wgpu::BindingType) -> Self {
        let binding = self.entries.len() as u32;

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty,
            count: None,
        });

        self
    }

    pub fn texture(self) -> Self {
        self.push(wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        })
    }

    pub fn texture_with(
        self,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
        multisampled: bool,
    ) -> Self {
        self.push(wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled,
        })
    }

    pub fn sampler(self) -> Self {
        self.push(wgpu::BindingType::Sampler(
            wgpu::SamplerBindingType::Filtering,
        ))
    }

    pub fn sampler_with(self, ty: wgpu::SamplerBindingType) -> Self {
        self.push(wgpu::BindingType::Sampler(ty))
    }

    pub fn uniform(self) -> Self {
        self.push(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        })
    }

    pub fn storage(self, read_only: bool) -> Self {
        self.push(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        })
    }

    /// Turns the most recently added entry into a binding array.
    pub fn count(mut self, count: u32) -> Self {
        if let Some(last) = self.entries.last_mut() {
            last.count = NonZeroU32::new(count);
        }
        self
    }

    pub fn build(self) -> wgpu::BindGroupLayout {
        let gpu = GpuState::get();

        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.label,
                entries: &self.entries,
            })
    }
}

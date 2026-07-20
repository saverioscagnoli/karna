use crate::Buffer;
use crate::GpuState;

pub struct BindGroupBuilder<'a> {
    label: Option<&'static str>,
    layout: &'a wgpu::BindGroupLayout,
    entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(label: &'static str, layout: &'a wgpu::BindGroupLayout) -> Self {
        Self {
            label: Some(label),
            layout,
            entries: Vec::new(),
        }
    }

    fn push(mut self, resource: wgpu::BindingResource<'a>) -> Self {
        let binding = self.entries.len() as u32;
        self.entries
            .push(wgpu::BindGroupEntry { binding, resource });
        self
    }

    pub fn texture(self, view: &'a wgpu::TextureView) -> Self {
        self.push(wgpu::BindingResource::TextureView(view))
    }

    pub fn sampler(self, sampler: &'a wgpu::Sampler) -> Self {
        self.push(wgpu::BindingResource::Sampler(sampler))
    }

    /// Bind an entire buffer (uniform or storage — the layout decides)
    pub fn buffer<T>(self, buffer: &'a Buffer<T>) -> Self {
        self.push(buffer.wgpu().as_entire_binding())
    }

    /// Bind a sub-range of a buffer
    pub fn buffer_range(
        self,
        buffer: &'a wgpu::Buffer,
        offset: u64,
        size: Option<std::num::NonZeroU64>,
    ) -> Self {
        self.push(wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset,
            size,
        }))
    }

    /// Binding array of texture views (pairs with `.count()` on the layout)
    pub fn texture_array(self, views: &'a [&'a wgpu::TextureView]) -> Self {
        self.push(wgpu::BindingResource::TextureViewArray(views))
    }

    /// Binding array of samplers
    pub fn sampler_array(self, samplers: &'a [&'a wgpu::Sampler]) -> Self {
        self.push(wgpu::BindingResource::SamplerArray(samplers))
    }

    pub fn build(self) -> wgpu::BindGroup {
        let gpu = GpuState::get();

        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout: self.layout,
            entries: &self.entries,
        })
    }
}

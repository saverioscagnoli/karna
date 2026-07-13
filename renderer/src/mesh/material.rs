use assets::AssetsRead;
use assets::Image;
use utils::Handle;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Blend {
    Opaque,
    Alpha,
    Additive,
}

impl Blend {
    fn to_wgpu(self) -> wgpu::BlendState {
        match self {
            Blend::Opaque => wgpu::BlendState::REPLACE,
            Blend::Alpha => wgpu::BlendState::ALPHA_BLENDING,
            Blend::Additive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topology {
    Triangles,
    Lines,
    Points,
}

impl Topology {
    fn to_wgpu(self) -> wgpu::PrimitiveTopology {
        match self {
            Topology::Triangles => wgpu::PrimitiveTopology::TriangleList,
            Topology::Lines => wgpu::PrimitiveTopology::LineList,
            Topology::Points => wgpu::PrimitiveTopology::PointList,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    #[doc(hidden)]
    pub pipeline_desc: gpu::PipelineDesc,

    uniforms: MaterialUniforms,
    uniform_buffer: gpu::Buffer<MaterialUniforms>,
    uniform_bind_group: wgpu::BindGroup,
}

#[derive(Clone, Copy)]
pub enum MaterialKind {
    /// Standard PBR-ish lit material.
    Standard,
    /// Unlit, just base color/texture.
    Unlit,
}

impl MaterialKind {
    fn shader(self) -> &'static str {
        match self {
            MaterialKind::Standard => "mesh-3d",
            MaterialKind::Unlit => "mesh-3d",
        }
    }

    pub fn default_blend(self) -> Blend {
        Blend::Opaque
    }
}

impl Material {
    pub fn new(
        desc: MaterialDesc,
        assets: &AssetsRead,
        material_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let image = assets.get_image(desc.base_color_texture.unwrap_or(assets.white_handle()));

        let uniforms = MaterialUniforms {
            base_color: desc.base_color.into(),
            emissive: desc.emissive.into(),
            uv_rect: image.uv.into(),
            metallic: desc.metallic,
            roughness: desc.roughness,
            _pad: [0.0; 2],
        };

        let uniform_buffer = gpu::Buffer::new_filled(
            "material uniforms",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[uniforms],
        );

        let gpu = gpu::GpuState::get();
        let uniform_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("material uniform bind group"),
            layout: material_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.wgpu().as_entire_binding(),
            }],
        });

        Self {
            pipeline_desc: gpu::PipelineDesc {
                shader: desc.kind.shader(),
                vertex_layout: gpu::Vertex::desc(),
                blend: desc.blend.to_wgpu(),
                topology: desc.topology.to_wgpu(),
            },
            uniforms,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.uniform_bind_group
    }

    pub fn set_base_color(&mut self, color: math::Vector4<f32>) {
        // Rewrite just the uniform buffer. Assumes you store the
        // current uniforms so you can re-upload the whole struct.
        // Simplest: keep a cached copy of MaterialUniforms.
        self.uniforms.base_color = color.into();
        self.uniform_buffer.write(0, &[self.uniforms]);
    }
}

pub struct MaterialDesc {
    pub kind: MaterialKind,
    pub blend: Blend,
    pub topology: Topology,

    pub base_color_texture: Option<Handle<Image>>,
    pub base_color: math::Vector4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: math::Vector4<f32>,
}

impl MaterialDesc {
    pub fn standard() -> Self {
        Self::default()
    }

    pub fn unlit() -> Self {
        Self {
            kind: MaterialKind::Unlit,
            ..Self::default()
        }
    }

    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<math::Vector4<f32>>,
    {
        self.base_color = color.into();
        self
    }

    pub fn texture(mut self, tex: Handle<Image>) -> Self {
        self.base_color_texture = Some(tex);
        self
    }

    pub fn metallic_roughness(mut self, m: f32, r: f32) -> Self {
        self.metallic = m;
        self.roughness = r;
        self
    }

    pub fn transparent(mut self) -> Self {
        self.blend = Blend::Alpha;
        self
    }
}

impl Default for MaterialDesc {
    fn default() -> Self {
        Self {
            kind: MaterialKind::Standard,
            blend: Blend::Alpha,
            topology: Topology::Triangles,
            base_color_texture: None,
            base_color: math::Vector4::new(1.0, 1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 1.0,
            emissive: math::Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
struct MaterialUniforms {
    base_color: [f32; 4],
    emissive: [f32; 4],
    uv_rect: [f32; 4],
    metallic: f32,
    roughness: f32,
    _pad: [f32; 2],
}

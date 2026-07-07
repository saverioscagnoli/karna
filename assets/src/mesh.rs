use gpu::GpuState;
use gpu::Vertex;
use utils::Handle;
use utils::SlotMap;

use crate::Image;
use crate::image::TextureAtlas;

pub struct Geometry {
    vertex_buffer: gpu::Buffer<Vertex>,
    index_buffer: gpu::Buffer<u32>,
    index_count: u32,
}

impl Geometry {
    pub fn new(vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = gpu::Buffer::new_filled(
            "geometry vertex buffer",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
            vertices,
        );
        let index_buffer = gpu::Buffer::new_filled(
            "geometry index buffer",
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_SRC,
            indices,
        );

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn vertex_buffer(&self) -> &gpu::Buffer<Vertex> {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &gpu::Buffer<u32> {
        &self.index_buffer
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn cube() -> (Vec<Vertex>, Vec<u32>) {
        Self::cube_sized(1.0)
    }

    pub fn cube_sized(size: f32) -> (Vec<Vertex>, Vec<u32>) {
        let h = size * 0.5;
        let white = math::Vector4::new(1.0, 1.0, 1.0, 1.0);

        // 4 verts per face, ordered so uv (0,0)-(1,1) maps consistently
        let mut positions = [
            // +X face
            [h, -h, -h],
            [h, h, -h],
            [h, h, h],
            [h, -h, h],
            // -X face
            [-h, -h, h],
            [-h, h, h],
            [-h, h, -h],
            [-h, -h, -h],
            // +Y face
            [-h, h, -h],
            [-h, h, h],
            [h, h, h],
            [h, h, -h],
            // -Y face
            [-h, -h, h],
            [-h, -h, -h],
            [h, -h, -h],
            [h, -h, h],
            // +Z face
            [-h, -h, h],
            [h, -h, h],
            [h, h, h],
            [-h, h, h],
            // -Z face
            [h, -h, -h],
            [-h, -h, -h],
            [-h, h, -h],
            [h, h, -h],
        ];

        let uvs = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];

        let vertices: Vec<Vertex> = positions
            .iter_mut()
            .enumerate()
            .map(|(i, p)| {
                let uv = uvs[i % 4];
                Vertex::new(
                    math::Vector3::new(p[0], p[1], p[2]),
                    white,
                    math::Vector2::new(uv[0], uv[1]),
                )
            })
            .collect();

        // two triangles per face, 6 faces
        let mut indices = Vec::with_capacity(36);
        for face in 0..6u32 {
            let base = face * 4;
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        (vertices, indices)
    }
}

pub struct Material {
    #[doc(hidden)]
    pub _pipeline_desc: gpu::PipelineDesc,

    uniform_buffer: gpu::Buffer<MaterialUniforms>,
    uniform_bind_group: wgpu::BindGroup,
}

impl Material {
    fn new(desc: MaterialDesc, atlas: &TextureAtlas, material_bgl: &wgpu::BindGroupLayout) -> Self {
        let image = atlas
            .images
            .get(desc.base_color_texture.unwrap_or(atlas.white))
            .expect("invalid base_color_texture handle");

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
            _pipeline_desc: gpu::PipelineDesc {
                shader: desc.shader,
                vertex_layout: gpu::Vertex::desc(),
                blend: desc.blend,
                topology: desc.topology,
            },
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.uniform_bind_group
    }
}

pub struct MaterialDesc {
    pub shader: &'static str,
    pub blend: wgpu::BlendState,
    pub topology: wgpu::PrimitiveTopology,

    pub base_color_texture: Option<Handle<Image>>,
    pub base_color: math::Vector4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: math::Vector4<f32>,
}

impl Default for MaterialDesc {
    fn default() -> Self {
        Self {
            shader: "mesh-3d",
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
            base_color_texture: None, // resolved to white_handle() in Material::new if unset — see below
            base_color: math::Vector4::new(1.0, 1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 1.0,
            emissive: math::Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MaterialUniforms {
    base_color: [f32; 4],
    emissive: [f32; 4],
    uv_rect: [f32; 4], // image.uv, so the shader knows where in the atlas to sample
    metallic: f32,
    roughness: f32,
    _pad: [f32; 2],
}

pub struct Meshes {
    geometries: SlotMap<Geometry>,
    materials: SlotMap<Material>,
    material_bgl: wgpu::BindGroupLayout,
}

impl Meshes {
    pub fn new() -> Self {
        let gpu = GpuState::get();
        let material_bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("material bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        Self {
            geometries: SlotMap::new(),
            materials: SlotMap::new(),
            material_bgl,
        }
    }

    pub fn create_geometry(&mut self, vertices: &[Vertex], indices: &[u32]) -> Handle<Geometry> {
        self.geometries.insert(Geometry::new(vertices, indices))
    }

    pub fn material_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.material_bgl
    }

    pub fn create_material(
        &mut self,
        desc: MaterialDesc,
        atlas: &TextureAtlas,
    ) -> Handle<Material> {
        let material = Material::new(desc, atlas, &self.material_bgl);
        self.materials.insert(material)
    }

    pub fn get_geometry(&self, geometry: Handle<Geometry>) -> &Geometry {
        self.geometries
            .get(geometry)
            .expect("Failed to get geometry")
    }

    pub fn get_material(&self, material: Handle<Material>) -> &Material {
        self.materials
            .get(material)
            .expect("Failed to get material")
    }
}

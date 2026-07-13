use std::mem;

use gpu::GpuState;
use utils::SlotMap;

use crate::Layouts;
use crate::Material;
use crate::Mesh;
use crate::mesh::Geometry;

/// One slot per mesh in the shared transform buffer. Padded to 256 bytes,
/// the maximum `min_uniform_buffer_offset_alignment` WebGPU allows, so
/// slot offsets are always valid dynamic offsets.
#[repr(C)]
#[derive(Clone, Copy)]
struct TransformUniform {
    matrix: [[f32; 4]; 4],
    _pad: [[f32; 4]; 12],
}

impl TransformUniform {
    fn new(matrix: math::Matrix4<f32>) -> Self {
        Self {
            matrix: matrix.as_array(),
            _pad: [[0.0; 4]; 12],
        }
    }
}

pub struct RetainedRenderer {
    pub meshes: SlotMap<Mesh>,

    transforms: gpu::Buffer<TransformUniform>,
    transforms_bg: Option<wgpu::BindGroup>,
    transforms_bg_capacity: usize,
}

impl RetainedRenderer {
    pub fn new() -> Self {
        Self {
            meshes: SlotMap::new(),
            transforms: gpu::Buffer::new_with_capacity(
                "retained transform buffer",
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                0,
            ),
            transforms_bg: None,
            transforms_bg_capacity: 0,
        }
    }

    pub fn present<'rp>(
        &'rp mut self,
        geometries: &'rp SlotMap<Geometry>,
        materials: &'rp SlotMap<Material>,
        pass: &mut wgpu::RenderPass<'rp>,
        pipelines: &gpu::PipelineCache,
        layouts: &Layouts,
        format: wgpu::TextureFormat,
    ) {
        if self.meshes.is_empty() {
            return;
        }

        let uniforms: Vec<TransformUniform> = self
            .meshes
            .values()
            .map(|mesh| TransformUniform::new(mesh.transform.matrix()))
            .collect();

        self.transforms.write_all(&uniforms);

        // Growing the buffer replaces the underlying wgpu buffer, which
        // invalidates any bind group pointing at the old one.
        if self.transforms_bg.is_none()
            || self.transforms_bg_capacity != self.transforms.capacity()
        {
            let gpu = GpuState::get();

            self.transforms_bg = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("retained transform bind group"),
                layout: &layouts.transform,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.transforms.wgpu(),
                        offset: 0,
                        size: wgpu::BufferSize::new(mem::size_of::<[[f32; 4]; 4]>() as u64),
                    }),
                }],
            }));

            self.transforms_bg_capacity = self.transforms.capacity();
        }

        let transforms_bg = self.transforms_bg.as_ref().unwrap();

        for (i, mesh) in self.meshes.values().enumerate() {
            let Some(geometry) = geometries.get(mesh.geometry) else {
                continue;
            };

            let Some(material) = materials.get(mesh.material) else {
                continue;
            };

            let pipeline = pipelines.get_or_create(
                material.pipeline_desc.clone(),
                format,
                &layouts.as_mesh_array(),
            );

            let offset = (i * mem::size_of::<TransformUniform>()) as u32;

            pass.set_pipeline(&pipeline);
            pass.set_bind_group(2, material.bind_group(), &[]);
            pass.set_bind_group(3, transforms_bg, &[offset]);
            pass.set_vertex_buffer(0, geometry.vertex_buffer().slice_all());
            pass.set_index_buffer(geometry.index_buffer().slice_all(), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..geometry.index_count(), 0, 0..1);
        }
    }
}

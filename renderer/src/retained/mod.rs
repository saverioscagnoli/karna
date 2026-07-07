use assets::AssetServerGuard;
use assets::Material;
use gpu::GpuState;
use gpu::PipelineCache;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::retained::mesh::Mesh;

pub mod handle;
pub mod mesh;

const MAX_INSTANCES: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy)]
struct TransformUniform {
    transform: [[f32; 4]; 4], // 64 bytes
    _pad: [f32; 48],          // pad to 256 bytes
}

pub struct RetainedRenderer {
    meshes: SlotMap<Mesh>,
    transform_bgl: wgpu::BindGroupLayout,
    transform_buffer: gpu::Buffer<TransformUniform>,
    transform_bind_group: wgpu::BindGroup,
}

impl RetainedRenderer {
    pub fn new(transform_bgl: &wgpu::BindGroupLayout) -> Self {
        let transform_buffer = gpu::Buffer::new_with_capacity(
            "transform buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            MAX_INSTANCES,
        );

        let transform_bind_group = Self::make_bind_group(transform_bgl, &transform_buffer);

        Self {
            meshes: SlotMap::new(),
            transform_bgl: transform_bgl.clone(),
            transform_buffer,
            transform_bind_group,
        }
    }

    pub fn create_transform_bgl() -> wgpu::BindGroupLayout {
        let gpu = GpuState::get();
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("transform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }

    fn make_bind_group(
        bgl: &wgpu::BindGroupLayout,
        buffer: &gpu::Buffer<TransformUniform>,
    ) -> wgpu::BindGroup {
        let gpu = gpu::GpuState::get();
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transform bind group"),
            layout: bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                // bind a fixed-size window; offset supplied per-draw
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer.wgpu(),
                    offset: 0,
                    size: Some(std::num::NonZeroU64::new(256).unwrap()),
                }),
            }],
        })
    }

    pub fn add(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.meshes.insert(mesh)
    }

    pub fn get(&self, mesh: Handle<Mesh>) -> &Mesh {
        self.meshes.get(mesh).expect("Failed to get mesh")
    }

    pub fn get_mut(&mut self, mesh: Handle<Mesh>) -> &mut Mesh {
        self.meshes.get_mut(mesh).expect("Failed to get mesh")
    }

    pub fn remove(&mut self, mesh: Handle<Mesh>) -> Mesh {
        self.meshes.remove(mesh).expect("Failed to remove mesh")
    }

    pub fn present<'a>(
        &'a mut self,
        rp: &mut wgpu::RenderPass<'a>,
        pipelines: &PipelineCache,
        assets: &AssetServerGuard<'a>,
    ) {
        // group instances by material for batching
        let mut by_material: FastHashMap<Handle<Material>, Vec<Handle<Mesh>>> =
            FastHashMap::default();
        for (handle, instance) in self.meshes.iter() {
            by_material
                .entry(instance.material)
                .or_default()
                .push(handle);
        }

        let ordered: Vec<Handle<Mesh>> = by_material.values().flatten().copied().collect();

        // grow + rebuild bind group if we have more instances than capacity
        if ordered.len() > self.transform_buffer.capacity() {
            self.transform_buffer.resize(ordered.len() * 2);
            self.transform_bind_group =
                Self::make_bind_group(&self.transform_bgl, &self.transform_buffer);
        }

        // upload all transforms in one shot, padded to 256B each
        let transforms: Vec<TransformUniform> = ordered
            .iter()
            .map(|h| TransformUniform {
                transform: self
                    .meshes
                    .get(*h)
                    .expect("mesh vanished")
                    .transform
                    .as_array(),
                _pad: [0.0; 48],
            })
            .collect();

        self.transform_buffer.write_all(&transforms);

        // NOTE: groups 0 (camera) and 1 (atlas) are already bound by the
        // caller (Renderer::_present) before this layer's present() runs —
        // do not rebind them here.

        let mut offset_index = 0u32;
        for (material_h, instances) in &by_material {
            let material = assets.get_material(*material_h);
            rp.set_pipeline(pipelines.get_pipeline(&material._pipeline_desc));
            rp.set_bind_group(2, material.bind_group(), &[]);

            for &mesh_h in instances {
                let instance = self.meshes.get(mesh_h).expect("mesh vanished");
                let geometry = assets.get_geometry(instance.geometry);

                let byte_offset = offset_index * 256;
                rp.set_bind_group(3, &self.transform_bind_group, &[byte_offset]);

                rp.set_vertex_buffer(0, geometry.vertex_buffer().slice_all());
                rp.set_index_buffer(
                    geometry.index_buffer().slice_all(),
                    wgpu::IndexFormat::Uint32,
                );
                rp.draw_indexed(0..geometry.index_count(), 0, 0..1);

                offset_index += 1;
            }
        }
    }
}

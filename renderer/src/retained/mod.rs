use std::mem;

use utils::Handle;
use utils::SlotMap;

use crate::Geometry;
use crate::Layouts;
use crate::Material;
use crate::Mesh;

#[repr(C)]
#[derive(Clone, Copy)]
struct InstanceData {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

struct DrawItem {
    material: Handle<Material>,
    geometry: Handle<Geometry>,
    matrix: math::Matrix4<f32>,
    color: [f32; 4],
}

pub struct RetainedRenderer {
    pub meshes: SlotMap<Mesh>,
    instances: gpu::Buffer<InstanceData>,
}

impl RetainedRenderer {
    pub fn new() -> Self {
        Self {
            meshes: SlotMap::new(),
            instances: gpu::Buffer::new_with_capacity(
                "retained instance buffer",
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                0,
            ),
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
        // 1. Collect draws, resolving handles up front so stale meshes
        //    never occupy an instance slot (keeps indices contiguous).
        let mut draws: Vec<DrawItem> = self
            .meshes
            .values()
            .filter(|m| geometries.get(m.geometry).is_some() && materials.get(m.material).is_some())
            .map(|m| DrawItem {
                material: m.material,
                geometry: m.geometry,
                matrix: m.transform.matrix(),
                // Linearize here: the shader multiplies instance color
                // directly, in linear space, without decoding it.
                color: m.color.to_linear_array(),
            })
            .collect();

        if draws.is_empty() {
            return;
        }

        // 2. Bucket: material first (pipeline + bind group cost),
        //    geometry second (vertex/index buffer cost).
        draws.sort_unstable_by_key(|item| (item.material, item.geometry));

        // 3. Upload matrices in bucketed order.
        let instance_data: Vec<InstanceData> = draws
            .iter()
            .map(|item| InstanceData {
                model: item.matrix.as_array(),
                color: item.color,
            })
            .collect();
        self.instances.write_all(&instance_data);

        // 4. Walk runs of identical (material, geometry) and emit one
        //    draw per run.
        let stride = mem::size_of::<InstanceData>() as u64;
        let mut start = 0;
        while start < draws.len() {
            let mat_h = draws[start].material;
            let geo_h = draws[start].geometry;
            let mut end = start + 1;
            while end < draws.len() && draws[end].material == mat_h && draws[end].geometry == geo_h
            {
                end += 1;
            }

            let material = materials.get(mat_h).unwrap();
            let geometry = geometries.get(geo_h).unwrap();

            let pipeline = pipelines.get_or_create(
                material.pipeline_desc.clone(),
                format,
                &layouts.as_mesh_array(),
            );

            pass.set_pipeline(&pipeline);
            pass.set_bind_group(2, material.bind_group(), &[]);
            pass.set_vertex_buffer(0, geometry.vertex_buffer().slice_all());
            pass.set_vertex_buffer(
                1,
                self.instances
                    .wgpu()
                    .slice(start as u64 * stride..end as u64 * stride),
            );
            pass.set_index_buffer(
                geometry.index_buffer().slice_all(),
                wgpu::IndexFormat::Uint32,
            );
            pass.draw_indexed(0..geometry.index_count(), 0, 0..(end - start) as u32);

            start = end;
        }
    }
}

use std::ops::Deref;
use std::ops::DerefMut;

use utils::Handle;

use crate::assets::Assets;
use crate::render::geometry::Geometry;
use crate::render::geometry::GeometryRegistry;
use crate::render::material::Material;
use crate::render::material::MaterialRegistry;
use crate::render::transform::Transform;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModelPacket {
    pub model: math::Matrix4<f32>,
    pub normal_matrix: math::Matrix4<f32>,
}

impl From<&Transform> for ModelPacket {
    fn from(t: &Transform) -> Self {
        Self {
            model: t.matrix(),
            normal_matrix: t.normal_matrix(),
        }
    }
}

pub struct Mesh {
    pub geometry: Handle<Geometry>,
    pub material: Handle<Material>,
    pub transform: Transform,
    pub visible: bool,
}

impl Mesh {
    pub fn new(geometry: Handle<Geometry>, material: Handle<Material>) -> Self {
        Self {
            geometry,
            material,
            transform: Transform::default(),
            visible: true,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn position(&self) -> &math::Vector3<f32> {
        &self.transform.position
    }

    pub fn position_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.transform.position
    }

    pub fn set_position<P>(&mut self, position: P)
    where
        P: Into<math::Vector3<f32>>,
    {
        self.transform.position = position.into();
    }

    pub fn scale(&self) -> &math::Vector3<f32> {
        &self.transform.scale
    }

    pub fn scale_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.transform.scale
    }

    pub fn set_scale<S>(&mut self, scale: S)
    where
        S: Into<math::Vector3<f32>>,
    {
        self.transform.scale = scale.into();
    }

    pub fn rotation(&self) -> &math::Vector3<f32> {
        &self.transform.rotation
    }

    pub fn rotation_mut(&mut self) -> &mut math::Vector3<f32> {
        &mut self.transform.rotation
    }

    pub fn set_rotation<R>(&mut self, rotation: R)
    where
        R: Into<math::Vector3<f32>>,
    {
        self.transform.rotation = rotation.into();
    }

    pub fn rotate_x(&mut self, x: f32) {
        self.transform.rotation.x += x;
    }

    pub fn rotate_y(&mut self, y: f32) {
        self.transform.rotation.y += y;
    }

    pub fn rotate_z(&mut self, z: f32) {
        self.transform.rotation.z += z;
    }

    pub fn rotate<R>(&mut self, rotation: R)
    where
        R: Into<math::Vector3<f32>>,
    {
        self.transform.rotation += rotation.into();
    }
}

#[derive(Clone, Copy)]
pub struct DrawItem {
    pub key: u64,
    pub geometry: Handle<Geometry>,
    pub material: Handle<Material>,
    pub model: ModelPacket,
}

pub fn sort_key(
    pass_id: u16,
    bind_id: u16,
    material: Handle<Material>,
    geometry: Handle<Geometry>,
) -> u64 {
    ((pass_id as u64) << 48)
        | ((bind_id as u64) << 32)
        | ((material.index() as u64 & 0xFFFF) << 16)
        | (geometry.index() as u64 & 0xFFFF)
}

pub struct MeshRenderer;

impl MeshRenderer {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &self,
        gpu: &gpu::Gpu,
        pipelines: &gpu::PipelineCache,
        cmd: &gpu::CommandBuffer,
        rpass: &gpu::RenderPass,
        assets: &Assets,
        materials: &MaterialRegistry,
        geometries: &GeometryRegistry,
        camera: &crate::render::camera::CameraPacket,
        format: gpu::TextureFormat,
        items: &[DrawItem],
    ) {
        if items.is_empty() {
            return;
        }

        let mut cur_pass = u16::MAX;
        let mut cur_material = Handle::INVALID;
        let mut cur_geometry = Handle::INVALID;

        for item in items {
            let Some(geometry) = geometries.get(item.geometry) else {
                continue;
            };

            let material = materials.get(item.material);

            if material.pass_id != cur_pass {
                let desc = materials
                    .pass(material.pass_id)
                    .pipeline_desc(geometry.layout, format);

                rpass.bind_graphics_pipeline(pipelines.get(&desc));

                cur_pass = material.pass_id;
                cur_material = Handle::INVALID;

                cmd.push_vertex_uniform_data(0, camera);
            }

            if item.material != cur_material {
                let bindings: Vec<_> = materials
                    .pages(material.bind_id)
                    .iter()
                    .filter_map(|&page| assets.page_texture(page))
                    .map(|texture| texture.binding(gpu))
                    .collect();

                if bindings.is_empty() {
                    continue;
                }

                rpass.bind_fragment_samplers(0, &bindings);
                gpu::push_fragment_uniform_bytes(cmd, 0, &material.uniforms);

                cur_material = item.material;
            }

            if item.geometry != cur_geometry {
                let Some((vertices, indices)) = geometry.bindings() else {
                    continue;
                };

                rpass.bind_vertex_buffers(0, &[vertices]);
                rpass.bind_index_buffer(&indices, gpu::IndexElementSize::_32BIT);

                cur_geometry = item.geometry;
            }

            cmd.push_vertex_uniform_data(1, &item.model);
            rpass.draw_indexed_primitives(geometry.index_count(), 1, 0, 0, 0);
        }
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self::new()
    }
}

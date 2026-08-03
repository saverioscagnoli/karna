use utils::Handle;
use utils::SlotMap;

use crate::Color;
use crate::Image;
use crate::assets::Assets;
use crate::render::camera::Camera;
use crate::render::camera::CameraPacket;
use crate::render::camera::Projection;
use crate::render::geometry::Geometry;
use crate::render::geometry::GeometryRegistry;
use crate::render::layer::Layer;
use crate::render::material::Material;
use crate::render::material::MaterialDesc;
use crate::render::material::MaterialRegistry;
use crate::render::mesh::DrawItem;
use crate::render::mesh::Mesh;
use crate::render::mesh::ModelPacket;
use crate::render::mesh::sort_key;
use crate::render::vertex::MeshVertex;

#[derive(Debug, Clone, Copy)]
pub struct ViewConfig {
    pub camera: Option<Handle<Camera>>,
    pub enabled: bool,
}

impl ViewConfig {
    fn new() -> Self {
        Self {
            camera: None,
            enabled: true,
        }
    }
}

pub struct RenderWorld {
    pub clear_color: Color,
    cameras: SlotMap<Camera>,
    views: [ViewConfig; 3],
    pub(crate) geometries: GeometryRegistry,
    pub(crate) materials: MaterialRegistry,
    meshes: SlotMap<Mesh>,
}

impl Default for RenderWorld {
    fn default() -> Self {
        Self {
            clear_color: Color::Black,
            cameras: SlotMap::new(),
            views: [ViewConfig::new(); 3],
            geometries: GeometryRegistry::new(),
            materials: MaterialRegistry::new(),
            meshes: SlotMap::new(),
        }
    }
}

impl RenderWorld {
    fn resolve(&self, layer: Layer) -> Option<Handle<Camera>> {
        let view = match layer {
            Layer::World => self.views[0],
            Layer::Ui => self.views[1],
            Layer::Debug => self.views[1],
        };

        match (view.camera, layer) {
            (Some(handle), _) => Some(handle),
            (None, Layer::Debug) => self.views[0].camera,
            (None, _) => None,
        }
    }

    pub(crate) fn view_enabled(&self, layer: Layer) -> bool {
        let view = match layer {
            Layer::World => self.views[0],
            Layer::Ui => self.views[1],
            Layer::Debug => self.views[1],
        };

        view.enabled
    }

    pub(crate) fn view_needs_depth(&self, layer: Layer) -> bool {
        self.resolve(layer)
            .and_then(|h| self.cameras.get(h))
            .map(|c| c.is_perspective())
            .unwrap_or(false)
    }

    pub(crate) fn refresh_materials(&mut self, assets: &Assets) {
        self.materials.refresh(assets);
    }

    pub(crate) fn update_cameras(&mut self, viewport: math::Size<u32>) {
        for camera in self.cameras.values_mut() {
            camera.update(viewport);
        }
    }

    pub(crate) fn camera_packet(&self, layer: Layer, viewport: math::Size<u32>) -> CameraPacket {
        match self.resolve(layer).and_then(|h| self.cameras.get(h)) {
            Some(camera) => camera.packet(),
            None => CameraPacket {
                view_projection: Projection::standard_2d(viewport).matrix(),
            },
        }
    }

    /// Flatten every visible mesh into sorted draw items.
    pub(crate) fn extract_meshes(&self, items: &mut Vec<DrawItem>) {
        items.clear();

        for mesh in self.meshes.values() {
            if !mesh.visible {
                continue;
            }

            let material = self.materials.get(mesh.material);

            items.push(DrawItem {
                key: sort_key(
                    material.pass_id,
                    material.bind_id,
                    mesh.material,
                    mesh.geometry,
                ),
                geometry: mesh.geometry,
                material: mesh.material,
                model: ModelPacket {
                    model: mesh.transform.matrix(),
                    normal_matrix: mesh.transform.normal_matrix(),
                },
            });
        }

        items.sort_unstable_by_key(|item| item.key);
    }
}

pub struct SceneRef<'a> {
    pub(crate) render: &'a mut RenderWorld,
}

impl<'a> SceneRef<'a> {
    pub fn clear_color(&self) -> Color {
        self.render.clear_color
    }

    pub fn clear_color_mut(&mut self) -> &mut Color {
        &mut self.render.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.render.clear_color = color.into();
    }

    pub fn add_camera(&mut self, camera: Camera) -> Handle<Camera> {
        self.render.cameras.insert(camera)
    }

    pub fn get_camera(&self, handle: Handle<Camera>) -> &Camera {
        self.render
            .cameras
            .get(handle)
            .expect("Failed to get camera")
    }

    pub fn get_camera_mut(&mut self, handle: Handle<Camera>) -> &mut Camera {
        self.render
            .cameras
            .get_mut(handle)
            .expect("Failed to get camera")
    }

    pub fn set_camera(&mut self, layer: Layer, camera: Handle<Camera>) {
        let index = match layer {
            Layer::World => 0,
            Layer::Ui => 1,
            Layer::Debug => 2,
        };

        self.render.views[index].camera = Some(camera);
    }

    pub fn add_geometry(&mut self, geometry: Geometry) -> Handle<Geometry> {
        self.render.geometries.insert(geometry)
    }

    pub fn create_geometry<L>(
        &mut self,
        label: L,
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
    ) -> Handle<Geometry>
    where
        L: AsRef<str>,
    {
        self.add_geometry(Geometry::new(label, vertices, indices))
    }

    pub fn add_material(&mut self, assets: &Assets, desc: MaterialDesc) -> Handle<Material> {
        self.render.materials.intern(assets, desc)
    }

    pub fn create_textured_material(
        &mut self,
        assets: &Assets,
        image: Handle<Image>,
        color: Color,
    ) -> Handle<Material> {
        self.add_material(assets, Self::tinted(image, color))
    }

    pub fn create_color_material(&mut self, assets: &Assets, color: Color) -> Handle<Material> {
        let white = assets.white_pixel_handle();

        self.create_textured_material(assets, white, color)
    }

    pub fn tinted(image: Handle<Image>, color: Color) -> MaterialDesc {
        let tint: math::Vector4<f32> = color.into();

        MaterialDesc::new()
            .with_uniform([tint.x, tint.y, tint.z, tint.w])
            .with_image(image)
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.render.meshes.insert(mesh)
    }

    pub fn get_mesh(&self, handle: Handle<Mesh>) -> &Mesh {
        self.render.meshes.get(handle).expect("Failed to get mesh")
    }

    pub fn get_mesh_mut(&mut self, handle: Handle<Mesh>) -> &mut Mesh {
        self.render
            .meshes
            .get_mut(handle)
            .expect("Failed to get mesh")
    }
}

#![allow(unused)]

use std::fmt::format;

use karna::App;
use karna::Context;
use karna::Scene;
use karna::SceneView;
use karna::WindowBuilder;
use karna::assets::Handle;
use karna::logging::Config;
use karna::math::Vector3;
use karna::render::Camera;
use karna::render::Color;
use karna::render::Draw;
use karna::render::Geometry;
use karna::render::Material;
use karna::render::Mesh;
use karna::render::MeshStorage;
use karna::render::MeshVertex;
use karna::render::Projection;
use karna::render::Transform;

struct CubeScene {
    meshes: MeshStorage,
    cubes: Vec<Mesh>,
    geometry: Handle<Geometry>,
    base_vertices: Vec<MeshVertex>,
    materials: [Handle<Material>; 2],
    colors: [[f32; 4]; 2],
    puff: f32,
    scale: f32,
}

impl Scene for CubeScene {
    fn load(&mut self, ctx: &mut Context, scene: &mut SceneView) {
        let mut camera = Camera::new(Projection::standard_3d(
            *ctx.window.size(),
            60.0,
            0.1,
            100.0,
        ));

        camera.position = Vector3::new(0.0, 3.0, -6.0);
        camera.target = Vector3::zero();
        *scene.camera_mut() = camera;

        let cube = Geometry::cube(1.0);
        self.base_vertices = cube.vertices().to_vec();
        self.geometry = self.meshes.add_geometry(cube);

        self.materials = [
            self.meshes.add_material(Material::standard(self.colors[0])),
            self.meshes.add_material(Material::standard(self.colors[1])),
        ];

        for i in 0..5 {
            let material = self.materials[i % 2];
            let x = (i as f32 - 2.0) * 1.8;

            self.cubes.push(Mesh::new(
                self.geometry,
                material,
                Transform::at(Vector3::new(x, 0.0, 0.0)),
            ));
        }
    }

    fn update(&mut self, ctx: &mut Context, scene: &mut SceneView) {
        let dt = ctx.time.delta();

        for (i, cube) in self.cubes.iter_mut().enumerate() {
            let speed = 0.6 + i as f32 * 0.25;
            let t = cube.transform_mut();

            t.rotation.y += dt * speed;
            t.rotation.x += dt * speed * 0.5;
        }
    }

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {
        draw.set_clear_color([0.08, 0.08, 0.1, 1.0]);

        let meshes = &mut self.meshes;
        let materials = self.materials;
        let geometry = self.geometry;
        let colors = &mut self.colors;
        let puff = &mut self.puff;
        let scale = &mut self.scale;
        let base_vertices = &self.base_vertices;

        draw.imgui(|ui| {
            ui.window("mesh controls")
                .size([320.0, 220.0], karna::imgui::Condition::FirstUseEver)
                .position([20.0, 40.0], karna::imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!("fps {}", ctx.time.fps()));
                    ui.text(format!("dt {}", ctx.time.delta()));

                    ui.separator();

                    if ui.color_edit4("color a", &mut colors[0]) {
                        meshes.material_mut(materials[0]).set_base_color(colors[0]);
                    }

                    if ui.color_edit4("color b", &mut colors[1]) {
                        meshes.material_mut(materials[1]).set_base_color(colors[1]);
                    }

                    ui.separator();

                    let puffed = ui.slider("puff", -0.3, 0.5, puff);
                    let scaled = ui.slider("scale", 0.2, 2.0, scale);

                    if puffed || scaled {
                        let vertices = meshes.geometry_mut(geometry).vertices_mut();

                        for (v, base) in vertices.iter_mut().zip(base_vertices) {
                            v.position = base.position * *scale + base.normal * *puff;
                        }
                    }
                });
        });

        for cube in &self.cubes {
            draw.mesh(&self.meshes, cube);
        }

        draw.set_color(Color::White);
        draw.debug_text(format!("fps: {}", ctx.time.fps()), 10.0, 10.0);
    }
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("mesh")
                .with_size((1280, 720))
                .with_resizable(true)
                .with_scene(
                    0,
                    CubeScene {
                        meshes: MeshStorage::new(),
                        cubes: Vec::new(),
                        geometry: Handle::INVALID,
                        base_vertices: Vec::new(),
                        materials: [Handle::INVALID; 2],
                        colors: [[1.0, 0.65, 0.0, 1.0], [0.2, 0.4, 1.0, 1.0]],
                        puff: 0.0,
                        scale: 1.0,
                    },
                )
                .with_active_scene(0),
        )
        .build()
        .run();
}

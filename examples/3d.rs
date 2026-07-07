use std::f32;

use assets::Geometry;
use assets::Material;
use imgui::ColorPreview::HalfAlpha;
use karna::App;
use karna::ContextRef;
use karna::ContextRefMut;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::MaterialDesc;
use karna::input::KeyCode;
use karna::math::Size;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use math::Matrix4;
use math::Vector3;
use renderer::Mesh;
use utils::Handle;

struct S {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    mesh: Handle<Mesh>,
    angle: f32,
}

impl Scene for S {
    fn load(&mut self, mut ctx: ContextRefMut) {
        if let Some(m) = ctx.monitors.current() {
            ctx.time.set_target_fps(m.refresh_rate());
        }

        let (v, i) = Geometry::cube_sized(1.0);
        let geom = ctx.assets.create_geometry(&v, &i);
        let mat = ctx.assets.create_material(MaterialDesc::default());
        let mesh = Mesh {
            geometry: geom,
            material: mat,
            transform: Matrix4::from_translation(Vector3::new(0.0, 0.0, 5.0)),
        };

        self.mesh = ctx.scene.add(mesh);
    }

    fn update(&mut self, mut ctx: ContextRefMut) {
        self.angle += ctx.time.delta() * f32::consts::PI; // radians/sec, tweak speed here

        let mesh = ctx.scene.get_mut(self.mesh);
        mesh.transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, 5.0))
            .matmul(&Matrix4::from_rotation_y(self.angle))
            .matmul(&Matrix4::from_rotation_z(self.angle))
    }

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {}
}

fn main() {
    karna::init_logging();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_scene(
                    "demo",
                    S {
                        pos: Vector2::new(50.0, 50.0),
                        vel: Vector2::zero(),
                        mesh: Handle::default(),
                        angle: 0.0,
                    },
                )
                .with_active_scene("demo"),
        )
        .build()
        .run();
}

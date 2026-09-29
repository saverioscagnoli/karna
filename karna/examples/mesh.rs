#![allow(unused)]

use karna::prelude::*;

const MESH_DEMO: SceneId = SceneId::new_str("mesh");

struct MeshDemo {
    cube: Handle<Mesh>,
}

impl Scene for MeshDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        let geo = ctx.assets.add_geometry(Geometry::cube(1.0));
        let mat = ctx
            .assets
            .add_material(Material::new().with_color(Color::RED));

        let mut camera = Camera::new(Projection::Perspective {
            fov: 60f32.to_radians(),
            aspect_ratio: ctx.window.aspect_ratio(),
            near: 0.1,
            far: 100.0,
        });

        camera.set_position(vec3!(2.0, 2.0, -4.0));
        camera.set_target(vec3!(0.0, 0.0, 0.0));
        ctx.scene.set_camera(Layer::WORLD, camera);

        Self {
            cube: ctx.scene.spawn(Mesh::new(geo, mat)),
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();
        let cube = ctx.scene.mesh_mut(self.cube);
        *cube.rotation_mut() *= Quaternion::from_euler(dt * 0.6, dt, 0.0);
        let material = cube.material();

        let t = ctx.time.elapsed_secs();
        let speed = 1.0;
        let mat = ctx.assets.material_mut(material);

        mat.color.r = (t * speed).sin() * 0.5 + 0.5;
        mat.color.g = (t * speed + 2.094).sin() * 0.5 + 0.5;
        mat.color.b = (t * speed + 4.189).sin() * 0.5 + 0.5;
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        let cube = ctx.scene.mesh(self.cube);

        draw.set_layer(Layer::UI);
        draw.print(
            format!("Mesh position: {:.3?}", cube.position()),
            10.0,
            10.0,
        );
        draw.print(
            format!("Mesh rotation: {:.3?}", cube.transform().rotation),
            10.0,
            30.0,
        );
    }
}

fn main() {
    AppBuilder::default()
        .with_window(
            WindowBuilder::new()
                .with_title("Mesh Demo")
                .with_size((1280, 720))
                .with_scene::<MeshDemo>(MESH_DEMO)
                .with_active_scene(MESH_DEMO),
        )
        .with_root("karna/examples/")
        .build()
        .run();
}

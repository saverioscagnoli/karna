use karna::{AppBuilder, Scene, WindowBuilder, input::KeyCode};
use math::Vector2;
use renderer::{Color, Geometry, Layer, Material, Mesh, MeshHandle, Transform};

struct Demo {
    rect1: MeshHandle,
}

impl Scene for Demo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.time.set_target_fps(120);
        ctx.render.set_clear_color(Color::Black);

        self.rect1 = ctx.render.add_mesh(Mesh::new(
            Geometry::rect((50.0, 50.0)),
            Material::new_color(Color::Red),
            Transform::new_2d([10.0, 10.0], 0.0, Vector2::ones()),
        ))
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let vel = 250.0;
        let rect1 = ctx.render.get_mesh_mut(self.rect1);

        if ctx.input.key_held(&KeyCode::KeyW) {
            *rect1.position_y_mut() -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            *rect1.position_x_mut() -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            *rect1.position_y_mut() += vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            *rect1.position_x_mut() += vel * ctx.time.delta()
        }

        if ctx.input.key_pressed(&KeyCode::Space) {
            *rect1.color_mut() = Color::random();
            ctx.render.toggle_wireframe();
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {}
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("demo window")
                .with_label("main")
                .with_resizable(false)
                .with_size((800, 600))
                .with_initial_scene(Demo {
                    rect1: MeshHandle::dummy(),
                }),
        )
        .build()
        .run();
}

use engine::input::KeyCode;
use karna::{
    App, Context, Scene,
    color::Color,
    define_mesh_from_gltf,
    math::{Vector3, Vector4},
    mesh::{Cube, Mesh, Rectangle},
};
use math::Vector;

define_mesh_from_gltf!(Duck, "Duck.gltf");

pub struct S {
    rect: Rectangle,
    rect_2: Rectangle,
    cube: Cube,
    vel: Vector3,
    duck: Duck,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        let vel = 5.0;

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y = -vel;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y = vel;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x = vel;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x = -vel;
        }

        self.rect.position.x += self.vel.x;
        self.rect.position.y += self.vel.y;

        self.vel.x *= 0.9;
        self.vel.y *= 0.9;

        self.cube
            .set_rotation(self.cube.rotation + Vector3::new(0.01, 0.01, 0.01));

        self.duck.rotation += Vector3::new(0.0, 0.0, 0.1);
    }

    fn fixed_update(&mut self, ctx: &mut Context) {}

    fn render(&self, ctx: &mut Context) {
        self.cube.render(&mut ctx.render);
        self.rect_2.render(&mut ctx.render);
        self.rect.render(&mut ctx.render);
        self.duck.render(&mut ctx.render);
    }
}

fn main() {
    App::new()
        .with_scene(
            "default",
            Box::new(S {
                cube: Cube::new()
                    .with_position_x(200.0)
                    .with_position_y(200.0)
                    .with_position_z(-100.0)
                    .with_color(Vector4::new(0.0, 1.0, 0.0, 1.0))
                    .with_scale(Vector3::new(70.0, 70.0, 70.0)),
                rect: Rectangle::new()
                    .with_position_x(10.0)
                    .with_position_y(10.0)
                    .with_position_z(-10.0)
                    .with_scale_x(50.0)
                    .with_scale_y(50.0)
                    .with_color(Vector4::new(1.0, 0.0, 0.0, 1.0)),
                rect_2: Rectangle::new()
                    .with_position_x(10.0)
                    .with_position_y(10.0)
                    .with_scale_x(50.0)
                    .with_scale_y(50.0)
                    .with_position_z(-9.0)
                    .with_scale(Vector3::new(50.0, 50.0, 0.0))
                    .with_color(Vector4::new(1.0, 1.0, 1.0, 1.0)),
                vel: Vector3::zero(),
                duck: Duck::new()
                    .with_position_x(500.0)
                    .with_position_y(500.0)
                    .with_position_z(-100.0)
                    .with_rotation_x(180.0_f32.to_radians())
                    .with_scale([1.0, 1.0, 1.0].into())
                    .with_color(Color::Cyan),
            }),
        )
        .with_current_scene("default")
        .run();
}

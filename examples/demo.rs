#![allow(unused)]

//use karna::{
//    App, Scene, WindowBuilder,
//    input::KeyCode,
//    label,
//    render::{Color, Geometry, Material, Mesh, Transform},
//};
//use math::Vector2;
//
//struct S {
//    rect1: Mesh,
//    rect2: Mesh,
//    visible: bool,
//}
//
//impl Scene for S {
//    fn load(&mut self, ctx: &mut karna::Context) {
//        ctx.render.set_clear_color(Color::Black);
//        ctx.render
//            .load_image(label!("cat"), include_bytes!("assets/cat.jpg").to_vec());
//    }
//
//    fn update(&mut self, ctx: &mut karna::Context) {
//        let vel = 250.0;
//
//        if ctx.input.key_held(&KeyCode::KeyW) {
//            self.rect1.position_mut().y -= vel * ctx.time.delta();
//        }
//
//        if ctx.input.key_held(&KeyCode::KeyS) {
//            self.rect1.position_mut().y += vel * ctx.time.delta();
//        }
//
//        if ctx.input.key_held(&KeyCode::KeyA) {
//            self.rect1.position_mut().x -= vel * ctx.time.delta();
//        }
//
//        if ctx.input.key_held(&KeyCode::KeyD) {
//            self.rect1.position_mut().x += vel * ctx.time.delta();
//        }
//
//        if ctx.input.key_pressed(&KeyCode::Space) {
//            self.rect1.set_color(Color::random());
//        }
//
//        if ctx.input.key_pressed(&KeyCode::KeyJ) {
//            self.visible = !self.visible;
//        }
//
//        println!("fps: {} dt: {}", ctx.time.fps(), ctx.time.delta());
//    }
//
//    fn render(&mut self, ctx: &mut karna::Context) {
//        if self.visible {
//            ctx.render.draw_mesh(&self.rect1);
//        }
//
//        ctx.render.draw_mesh(&self.rect2);
//    }
//}
//
//struct DebugScene;
//
//impl Scene for DebugScene {
//    fn load(&mut self, ctx: &mut karna::Context) {}
//
//    fn update(&mut self, ctx: &mut karna::Context) {}
//
//    fn render(&mut self, ctx: &mut karna::Context) {
//        ctx.render.draw_texture_atlas();
//    }
//}
//
//fn main() {
//    App::builder()
//        .with_window(
//            WindowBuilder::new()
//                .with_label("main window")
//                .with_title("Karna demo")
//                .with_resizable(false)
//                .with_initial_scene(S {
//                    rect1: Mesh::new(
//                        Geometry::rect(50.0, 50.0),
//                        Material::new_color(Color::Red),
//                        Transform::default().with_position([50.0, 50.0]),
//                    ),
//                    rect2: Mesh::new(
//                        Geometry::rect(50.0, 50.0),
//                        Material::new_texture(label!("cat")),
//                        Transform::default().with_position([50.0, 150.0]),
//                    ),
//                    visible: true,
//                }),
//        )
//        .with_window(
//            WindowBuilder::new()
//                .with_label("secondary window")
//                .with_size((1024, 1024))
//                .with_title("Karna demo")
//                .with_resizable(false)
//                .with_initial_scene(DebugScene),
//        )
//        .build()
//        .run();
//}

use karna::{
    AppBuilder, Scene, WindowBuilder,
    input::KeyCode,
    render::{Color, Geometry, Material, Mesh},
};
use renderer::Transform;

struct S {
    rect1: Mesh,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.time.set_target_fps(120);
        ctx.render.set_clear_color(Color::Black);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let vel = 250.0;

        if ctx.input.key_held(&KeyCode::KeyW) {
            *self.rect1.position_y_mut() -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            *self.rect1.position_y_mut() += vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            *self.rect1.position_x_mut() -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            *self.rect1.position_x_mut() += vel * ctx.time.delta();
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_mesh(&self.rect1);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("demo window")
                .with_label("main")
                .with_resizable(false)
                .with_size((800, 600))
                .with_initial_scene(S {
                    rect1: Mesh::new(
                        Geometry::rect(50.0, 50.0),
                        Material::new_color(Color::Red),
                        Transform::default().with_position([50.0, 50.0]),
                    ),
                }),
        )
        .build()
        .run();
}

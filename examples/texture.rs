#![allow(unused)]
use karna::{App, Scene, WindowBuilder, label, render::Mesh};
use renderer::{Color, Geometry, Material, TextureKind, Transform};

struct ImageDemo {
    target: Mesh,
    rect: Mesh,
}

impl Scene for ImageDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.assets
            .load_image(label!("cat"), include_bytes!("assets/cat.jpg").to_vec());
    }

    fn update(&mut self, ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_mesh(&self.target);
        ctx.render.draw_mesh(&self.rect);
    }
}

struct AtlasDebug {
    target: Mesh,
}

impl Scene for AtlasDebug {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
    }

    fn update(&mut self, ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_mesh(&self.target);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_resizable(false)
                .with_initial_scene(ImageDemo {
                    target: Mesh::new(
                        Geometry::rect(200.0, 200.0),
                        Material::new_texture(TextureKind::Full(label!("cat"))),
                        Transform::default().with_position([10.0, 10.0]),
                    ),
                    rect: Mesh::new(
                        Geometry::rect(200.0, 200.0),
                        Material::new_color(Color::Pink),
                        Transform::default().with_position([250.0, 250.0]),
                    ),
                }),
        )
        .with_window(
            WindowBuilder::new()
                .with_label("texture atlas")
                .with_size((1024, 1024))
                .with_resizable(false)
                .with_initial_scene(AtlasDebug {
                    target: Mesh::new(
                        Geometry::rect(1024.0, 1024.0),
                        Material::new_texture(TextureKind::Full(label!("_atlas"))),
                        Transform::default(),
                    ),
                }),
        )
        .build()
        .run();
}

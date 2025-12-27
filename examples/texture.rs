#![allow(unused)]
use karna::{App, Scene, WindowBuilder, label, render::Mesh};
use renderer::{Color, Geometry, Layer, Material, MeshHandle, TextureKind, Transform};

struct ImageDemo {
    target: MeshHandle,
    rect: MeshHandle,
}

impl Scene for ImageDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
        ctx.assets
            .load_image(label!("cat"), include_bytes!("assets/cat.jpg").to_vec());

        self.target = ctx.render.add_mesh(
            Layer::World,
            Mesh::new(
                Geometry::rect((200.0, 200.0)),
                Material::new_texture(TextureKind::Full(label!("cat"))),
                Transform::default().with_position([10.0, 10.0, 0.0]),
            ),
        );

        self.rect = ctx.render.add_mesh(
            Layer::World,
            Mesh::new(
                Geometry::rect((150.0, 50.0)),
                Material::new_color(Color::Pink),
                Transform::default().with_position([250.0, 25.0, 0.0]),
            ),
        );
    }

    fn update(&mut self, ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {}
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_resizable(false)
                .with_initial_scene(ImageDemo {
                    target: MeshHandle::dummy(),
                    rect: MeshHandle::dummy(),
                }),
        )
        .build()
        .run();
}

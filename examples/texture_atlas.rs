use image::{Rgba, RgbaImage};
use karna::{
    AppBuilder, Context, Label, Scene, WindowBuilder, label,
    math::rng,
    render::{Color, Geometry, Material, Mesh, Transform},
};
use renderer::TextureKind;
use std::io::Cursor;

pub struct TextureAtlasDemo {
    atlas: Mesh,
}

impl Scene for TextureAtlasDemo {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render
            .set_clear_color(Color::rgb(30.0 / 255.0, 30.0 / 255.0, 40.0 / 255.0));

        for i in 0..500 {
            let width = rng(10..=70);
            let height = rng(10..=70);

            let r = rng(50..=255);
            let g = rng(50..=255);
            let b = rng(50..=255);

            let mut img = RgbaImage::new(width, height);
            for pixel in img.pixels_mut() {
                *pixel = Rgba([r, g, b, 255]);
            }

            let mut png_bytes = Vec::new();

            img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
                .expect("Failed to encode PNG");

            let texture_label = Label::new(&format!("rect_{}", i));

            ctx.assets.load_image(texture_label, png_bytes);
        }
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.draw_mesh(&self.atlas);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Texture Atlas Demo - Efficient Batch Loading")
                .with_size((1024, 1024))
                .with_resizable(false)
                .with_initial_scene(TextureAtlasDemo {
                    atlas: Mesh::new(
                        Geometry::rect(1024.0, 1024.0),
                        Material::new_texture(TextureKind::Full(label!("_atlas"))),
                        Transform::default(),
                    ),
                }),
        )
        .build()
        .run();
}

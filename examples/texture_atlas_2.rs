use image::{Rgba, RgbaImage};
use karna::{AppBuilder, Context, Label, Scene, WindowBuilder, math::rng, render::Color};
use std::io::Cursor;

pub struct TextureAtlasDemo;

impl Scene for TextureAtlasDemo {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render
            .set_clear_color(Color::rgb(30.0 / 255.0, 30.0 / 255.0, 40.0 / 255.0));

        for i in 0..500 {
            let width = rng(5..=64);
            let height = rng(5..=64);

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

            ctx.render.load_image(texture_label, png_bytes);
        }
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {
        // Visualize the packed texture atlas
        ctx.render.draw_texture_atlas();
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Texture Atlas Demo - Efficient Batch Loading")
                .with_size((1024, 1024))
                .with_resizable(false)
                .with_initial_scene(TextureAtlasDemo),
        )
        .build()
        .run();
}

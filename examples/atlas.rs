use karna::App;
use karna::Image;
use karna::KeyCode;
use karna::Scene;
use karna::WindowBuilder;
use karna::math::Size;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use math::rng;
use utils::Handle;

struct S {
    pos: Vector2,
    vel: Vector2,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        ctx.time.set_target_fps(120);
        ctx.assets.load_png(include_bytes!("images/cat.png"));
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {
        ctx.window.set_title(format!("fps: {}", ctx.time.fps()));

        let accel = 5000.0;
        let dt = ctx.time.delta();

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y -= accel * dt;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x -= accel * dt;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y += accel * dt;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x += accel * dt
        }

        self.vel *= 0.85f32.powf(60.0).powf(dt);
        self.pos += self.vel * dt;
    }

    fn draw(&self, _ctx: karna::ContextRef, draw: &mut Draw) {
        draw.set_color(Color::White);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);

        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.point(i as f32 * 10.0, j as f32 * 10.0);
            }
        }

        draw.set_color(Color::Magenta);
        draw.line_v([300.0, 100.0], self.pos + Vector2::new(25.0, 25.0));
    }
}

struct AtlasDemo {
    squares: Vec<Handle<Image>>,
}

impl Scene for AtlasDemo {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        for i in 0..200 {
            // Generate a 50x50 colored square using the `image` crate
            let r = ((i * 123) % 255) as u8;
            let g = ((i * 456) % 255) as u8;
            let b = ((i * 789) % 255) as u8;

            let img = image::ImageBuffer::from_pixel(
                rng(25..100),
                rng(25..100),
                image::Rgba([r, g, b, 255]),
            );

            // Encode to PNG bytes in memory
            let mut bytes = std::io::Cursor::new(Vec::new());
            img.write_to(&mut bytes, image::ImageFormat::Png).unwrap();

            let handle = ctx.assets.load_png(&bytes.into_inner());
            self.squares.push(handle);
        }
    }

    fn update(&mut self, _ctx: karna::ContextRefMut) {}

    fn draw(&self, _ctx: karna::ContextRef, draw: &mut Draw) {
        // Draw the whole atlas
        draw.atlas(0.0, 0.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_initial_scene(S {
                    pos: Vector2::new(50.0, 50.0),
                    vel: Vector2::zeros(),
                }),
        )
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1024, 1024))
                .with_title("Texture Atlas")
                .with_initial_scene(AtlasDemo {
                    squares: Vec::new(),
                }),
        )
        .build()
        .run();
}

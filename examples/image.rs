use karna::App;
use karna::Handle;
use karna::Image;
use karna::KeyCode;
use karna::Scene;
use karna::WindowBuilder;
use karna::math::Size;
use karna::render::Draw;
use math::Vector2;

struct S {
    pos: Vector2,
    image: Handle<Image>,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        let bytes = include_bytes!("images/tetsuo.png");

        self.image = ctx.assets.load_png(bytes);

        let duck_bytes = include_bytes!("images/duck-cursor.png");
        let duck_image = ctx.assets.load_png(duck_bytes);

        ctx.window.set_icon(duck_image);

        ctx.window.set_cursor_image(duck_image, 0, 0);
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {
        ctx.window.set_title(format!("fps: {}", ctx.time.fps()));

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.pos.y -= 10.0;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.pos.x -= 10.0;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.pos.y += 10.0;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.pos.x += 10.0;
        }
    }

    fn draw(&self, ctx: karna::ContextRef, draw: &mut Draw) {
        draw.image_v(self.image, self.pos);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_initial_scene(S {
                    pos: Vector2::zeros(),
                    image: Handle::default(),
                }),
        )
        .build()
        .run();
}

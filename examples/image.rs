use karna::App;
use karna::ContextMut;
use karna::Handle;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Image;
use karna::input::KeyCode;
use karna::math::Size;
use karna::render::Draw;
use math::Vector2;

struct S {
    pos: Vector2<f32>,
    image: Handle<Image>,
}

impl Scene for S {
    fn load(&mut self, mut ctx: ContextMut) {
        let bytes = include_bytes!("assets/tetsuo.png");

        self.image = ctx.assets.load_image(bytes);

        let duck_bytes = include_bytes!("assets/duck.png");
        let duck_image = ctx.assets.load_image(duck_bytes);

        ctx.window.set_icon(duck_image);
        ctx.window.set_custom_cursor(duck_image, 0, 0);
    }

    fn update(&mut self, ctx: ContextMut) {
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

    fn draw(&mut self, _ctx: karna::ContextRef, draw: &mut Draw) {
        draw.image_v(self.image, self.pos);
    }
}

fn main() {
    karna::init_logging();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_scene(
                    "image",
                    S {
                        pos: Vector2::zero(),
                        image: Handle::default(),
                    },
                )
                .with_active_scene("image"),
        )
        .build()
        .run();
}

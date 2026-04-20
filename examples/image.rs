use karna::App;
use karna::Handle;
use karna::Image;
use karna::Scene;
use karna::WindowBuilder;
use karna::math::Size;
use karna::render::Draw;

struct S {
    image: Handle<Image>,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        let bytes = include_bytes!("images/cat.png");
        ctx.assets.load_png(bytes);
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {
        ctx.window.set_title(format!("fps: {}", ctx.time.fps()));
    }

    fn draw(&self, ctx: karna::ContextRef, draw: &mut Draw) {}
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_initial_scene(S {
                    image: Handle::default(),
                }),
        )
        .build()
        .run();
}

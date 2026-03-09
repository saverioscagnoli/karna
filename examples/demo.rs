use karna::App;
use karna::Scene;
use karna::WindowBuilder;
use karna::math::Size;
use renderer::Color;

struct S;

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {}

    fn update(&mut self, ctx: karna::ContextRefMut) {
        println!("dt {}", ctx.time.delta());
    }

    fn draw<'a>(&'a self, ctx: karna::ContextRef, draw: &mut karna::Draw<'a>) {
        draw.set_color(Color::Red);
        draw.fill_rect(0.0, 0.0, 1280.0, 50.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_resizable(false)
                .with_initial_scene(S),
        )
        .build()
        .run();
}

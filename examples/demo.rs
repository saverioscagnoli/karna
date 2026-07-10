use karna::App;
use karna::ContextMut;
use karna::ContextRef;
use karna::Scene;
use karna::WindowBuilder;
use karna::render::Draw;

struct S;

impl Scene for S {
    fn load(&mut self, ctx: ContextMut) {}

    fn update(&mut self, ctx: ContextMut) {
        println!("dt {}", ctx.time.delta());
    }

    fn draw(&self, ctx: ContextRef, draw: &mut Draw) {
        draw.rect(10.0, 10.0, 50.0, 50.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo")
                .with_size((1280, 720))
                .with_scene("demo", S)
                .with_active_scene("demo"),
        )
        .build()
        .run();
}

use karna::{App, Scene};

struct S;

impl Scene for S {
    fn load(&mut self, _: &mut karna::Context) -> Result<(), (karna::LoadControlFlow, String)> {
        Ok(())
    }

    fn update(&mut self, context: &mut karna::Context) {
        println!("dt: {}", context.time.delta());
    }

    fn render(&mut self, _: &mut karna::Context) {}
}

fn main() {
    App::new("Hello", (800, 600))
        .unwrap()
        .with_scene("default", S)
        .with_initial_scene("default")
        .run()
        .unwrap();
}

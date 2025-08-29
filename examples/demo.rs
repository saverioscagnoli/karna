use karna::{App, Color, Context, Rect, Scene};

struct S {
    rect: Rect,
    rect_2: Rect,
}

impl Default for S {
    fn default() -> Self {
        Self {
            rect: Rect::new(0.0, 0.0, 50.0, 50.0).with_color(Color::Cyan),
            rect_2: Rect::new(200.0, 200.0, 50.0, 150.0),
        }
    }
}

impl Scene for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        // self.rect.pos.x += 100.0 * ctx.time.delta();
        // self.rect.pos.y += 75.0 * ctx.time.delta();

        self.rect_2.pos.y += 25.0 * ctx.time.delta();
    }

    fn render(&mut self, ctx: &mut Context) {
        self.rect.render(&mut ctx.render);
        self.rect_2.render(&mut ctx.render);
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene("default", S::default())
        .run()
        .expect("Failed to run application");
}

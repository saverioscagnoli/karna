use karna::{App, Color, Context, Rect, Scene};

struct S {
    rect: Rect,
    rect_2: Rect,
}

impl Default for S {
    fn default() -> Self {
        Self {
            rect: Rect::new([10, 10], 50.0).with_color(Color::Cyan),
            rect_2: Rect::new([100, 100], (100.0, 50.0)).with_color(Color::Magenta),
        }
    }
}

impl Scene for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        self.rect.position.x += 100.0 * ctx.time.delta();
        self.rect.position.y += 75.0 * ctx.time.delta();

        if self.rect.position.x > ctx.window.size().width as f32 {
            self.rect.position.x = -self.rect.size.width;
        }
        if self.rect.position.y > ctx.window.size().height as f32 {
            self.rect.position.y = -self.rect.size.height;
        }
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

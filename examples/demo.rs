use karna::{
    input::KeyCode,
    math::Vec2,
    render::{Color, Rect},
    App, Context, Scene,
};

struct RectScene {
    player: Rect,
    vel: Vec2,
    rects: Vec<Rect>,
}

impl RectScene {
    fn new() -> Self {
        Self {
            player: Rect::new([10, 10], 50.0).with_color(Color::RED),
            vel: Vec2::zero(),
            rects: vec![],
        }
    }
}

impl Scene for RectScene {
    fn load(&mut self, _ctx: &mut Context) {
        for i in 0..10 {
            for j in 0..5 {
                let rect = Rect::new([i as f32 * 60.0 + 200.0, j as f32 * 60.0 + 100.0], 50.0)
                    .with_color(Color::GREEN);
                self.rects.push(rect);
            }
        }
    }

    fn fixed_update(&mut self, ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.key_held(KeyCode::KeyW) {
            self.vel.y = -200.0;
        }

        if ctx.input.key_held(KeyCode::KeyS) {
            self.vel.y = 200.0;
        }

        if ctx.input.key_held(KeyCode::KeyA) {
            self.vel.x = -200.0;
        }

        if ctx.input.key_held(KeyCode::KeyD) {
            self.vel.x = 200.0;
        }

        self.player.position += self.vel * ctx.time.delta();
        self.vel *= 0.9;
    }

    fn render(&mut self, ctx: &mut Context) {
        for rect in &self.rects {
            rect.render(&mut ctx.render);
        }

        self.player.render(&mut ctx.render);
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene("default", RectScene::new())
        .run()
        .expect("Failed to run application");
}

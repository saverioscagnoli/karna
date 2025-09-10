use karna::{
    input::KeyCode,
    math::Vec2,
    render::{Color, Mesh, Rect},
    App, Context, Scene,
};
use math::Vec3;

struct RectScene {
    player: Rect,
    vel: Vec2,
    rects: Vec<Rect>,
}

impl RectScene {
    fn new() -> Self {
        Self {
            player: Rect::new([10, 10, 0], 50.0).with_color(Color::RED),
            vel: Vec2::zero(),
            rects: vec![],
        }
    }
}

impl Scene for RectScene {
    fn load(&mut self, _ctx: &mut Context) {
        for i in 0..10 {
            for j in 0..5 {
                let rect = Rect::new(
                    [i as f32 * 60.0 + 200.0, j as f32 * 60.0 + 100.0, 0.0],
                    50.0,
                )
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

        self.player
            .add_position(self.vel.extend(0.0) * ctx.time.delta().as_secs_f32());

        self.player
            .add_rotation(Vec3::new(0.0, 0.0, 1.0 * ctx.time.delta().as_secs_f32()));
        self.vel *= 0.9;
    }

    fn render(&mut self, ctx: &mut Context) {
        for rect in self.rects.iter_mut() {
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

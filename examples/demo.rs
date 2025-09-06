struct RectScene {}

impl RectScene {
    fn new() -> Self {
        Self {
            player: Rect::new(Vec2::new(400.0, 300.0), 50.0, 50.0, Color::YELLOW),
            vel: Vec2::zero(),
            rects: vec![
                Rect::new(Vec2::new(100.0, 100.0), 50.0, 30.0, Color::RED),
                Rect::new(Vec2::new(200.0, 150.0), 80.0, 40.0, Color::GREEN),
                Rect::new(Vec2::new(300.0, 200.0), 60.0, 60.0, Color::BLUE),
            ],
        }
    }
}

impl Scene for RectScene {
    fn load(&mut self, ctx: &mut Context) {}
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

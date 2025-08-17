use karna::{App, Color, Context, Scene};
use math::Vec2;

struct S {
    rect_color: Color,
}

impl Default for S {
    fn default() -> Self {
        Self {
            rect_color: Color::Cyan,
        }
    }
}

impl Scene for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.set_draw_color(Color::Cyan);

        for i in 0..20 {
            let rect_x = 300.0 + (i as f32 * 15.0);
            let rect_y = 50.0 + (ctx.time.elapsed() + i as f32 * 0.1).sin() * 20.0;

            self.rect_color = Color::rgb(0.2 + i as f32 * 0.04, 0.8, 0.3);
            ctx.render.set_draw_color(self.rect_color);
            ctx.render.fill_rect([rect_x, rect_y], (10.0, 10.0));
        }

        let mut wave_points = Vec::new();
        for i in 0..100 {
            let x = 400.0 + i as f32 * 2.0;
            let y = 300.0 + (ctx.time.elapsed() + i as f32 * 0.1).sin() * 30.0;
            wave_points.push(Vec2::new(x, y));
        }

        ctx.render.set_draw_color(Color::Red);
        ctx.render.draw_line_strip(&wave_points);
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene("default", S::default())
        .run()
        .expect("Failed to run application");
}

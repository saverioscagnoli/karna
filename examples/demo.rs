use karna::{AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder, input::KeyCode};
use math::Vector2;
use renderer::Color;

struct Demo {
    pos: Vector2,
    color: Color,
}

impl Scene for Demo {
    fn load(&mut self, ctx: &mut Context) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: &mut Context) {
        let vel = 250.0;

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.pos.y -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.pos.x -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.pos.y += vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.pos.x += vel * ctx.time.delta();
        }

        if ctx.input.key_pressed(&KeyCode::Space) {
            self.color = Color::random();
        }
    }

    fn render(&mut self, ctx: &RenderContext, draw: &mut Draw) {
        draw.set_draw_color(self.color);
        draw.fill_rect(self.pos.x, self.pos.y, 50.0, 50.0);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("demo window")
                .with_label("main")
                .with_resizable(false)
                .with_size((800, 600))
                .with_initial_scene(Demo {
                    pos: Vector2::new(10.0, 10.0),
                    color: Color::Red,
                }),
        )
        .build()
        .run();
}

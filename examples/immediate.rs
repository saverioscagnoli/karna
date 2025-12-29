use karna::{AppBuilder, Scene, WindowBuilder, input::KeyCode};
use math::Vector2;
use renderer::Color;
use utils::label;

struct ImmediateRenderingDemo {
    pos: Vector2,
    vel: Vector2,
}

impl Scene for ImmediateRenderingDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.time.set_target_fps(120);
        ctx.assets.load_font(
            label!("jetbrains mono"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf").to_vec(),
            16,
        );
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let vel = 250.0;

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y = -vel;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y = vel;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x = -vel;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x = vel;
        }

        self.pos += self.vel * ctx.time.delta();
        self.vel *= 0.9;
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_draw_color(Color::White);
        ctx.render.debug_text(
            &format!("FPS: {}\ndt: {}", ctx.time.fps(), ctx.time.delta()),
            10.0,
            10.0,
        );

        ctx.render.set_font(label!("jetbrains mono"));

        ctx.render
            .draw_text_v("This is jetbrains mono", [200.0, 200.0]);

        ctx.render.set_draw_color(Color::Red);
        ctx.render.fill_rect_v(self.pos, (50.0, 50.0));

        ctx.render.set_draw_color(Color::Cyan);

        ctx.render
            .fill_rect(self.pos.x, self.pos.y + 200.0, 50.0, 50.0);
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
                .with_initial_scene(ImmediateRenderingDemo {
                    pos: Vector2::new(10.0, 10.0),
                    vel: Vector2::new(0.0, 0.0),
                }),
        )
        .build()
        .run();
}

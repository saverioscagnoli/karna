use karna::{AppBuilder, Scene, WindowBuilder, input::KeyCode};
use math::Vector2;
use renderer::Color;
use utils::{Label, label};

struct ImmediateRenderingDemo {
    cat_texture: Label,
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

        ctx.assets
            .load_image(self.cat_texture, include_bytes!("assets/cat.jpg").to_vec());
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

        ctx.render.set_draw_color(Color::Magenta);
        ctx.render.stroke_rect_v([100.0, 10.0], (50.0, 50.0));

        ctx.render.set_draw_color(Color::Cyan);

        ctx.render.draw_line_v([300.0, 10.0], [100.0, 500.0]);

        ctx.render
            .fill_rect(self.pos.x, self.pos.y + 200.0, 50.0, 50.0);

        ctx.render.set_draw_color(Color::Purple);

        let pos = [500.0, 400.0];
        for i in 0..10 {
            for j in 0..10 {
                ctx.render
                    .draw_point(pos[0] + i as f32 * 10.0, pos[1] + j as f32 * 10.0);
            }
        }

        ctx.render.set_draw_color(Color::Green);
        ctx.render
            .draw_subimage_tinted(self.cat_texture, 100.0, 200.0, 0.0, 0.0, 400.0, 50.0);
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
                    cat_texture: label!("cat"),
                    pos: Vector2::new(10.0, 10.0),
                    vel: Vector2::new(0.0, 0.0),
                }),
        )
        .build()
        .run();
}

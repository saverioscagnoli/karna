use karna::{
    input::Key,
    math::Vec2,
    shaders::Uniform,
    shaders::{Shader, ShaderKind},
    traits::Scene,
    App, Context,
};
use sdl2::pixels::Color;

const SPEED: f32 = 250.0;

struct FirstScene {
    pos: Vec2,
    vel: Vec2,
}

impl Scene for FirstScene {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.load_shader(
            "crt",
            Shader::from_str(include_str!("./assets/vs.glsl"), ShaderKind::Vertex),
            Shader::from_str(include_str!("./assets/fs.glsl"), ShaderKind::Fragment),
        );
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.key_down(Key::W) {
            self.vel.y = -SPEED;
        }

        if ctx.input.key_down(Key::S) {
            self.vel.y = SPEED;
        }

        if ctx.input.key_down(Key::A) {
            self.vel.x = -SPEED;
        }

        if ctx.input.key_down(Key::D) {
            self.vel.x = SPEED;
        }

        self.pos += self.vel * ctx.time.delta();
        self.vel *= 0.9;

        ctx.render.set_shader_uniform(
            "elapsed",
            Uniform::Float(ctx.time.started_at().elapsed().as_secs_f32()),
        );
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_shader("crt");
        ctx.render.set_color(Color::CYAN);

        ctx.render.fill_rect(self.pos, (50, 50));
        ctx.render.set_color(Color::MAGENTA);
        ctx.render.draw_line((300, 10), (50, 500));
        ctx.render.set_color(Color::RGB(53, 101, 77));

        ctx.render.fill_text(ctx.time.fps(), (10, 10), Color::WHITE);
        ctx.render.reset_shader();
    }
}

fn main() {
    App::window("basic window", (800, 600)).run(FirstScene {
        pos: Vec2::ONE,
        vel: Vec2::ZERO,
    });
}

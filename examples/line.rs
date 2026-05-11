use karna::App;
use karna::Scene;
use karna::WindowBuilder;
use renderer::Color;

struct S;

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        let duck_bytes = include_bytes!("images/duck.png");
        let duck_image = ctx.assets.load_png(duck_bytes);

        ctx.window.set_icon(duck_image);

        ctx.window.set_cursor_image(duck_image, 16, 16);
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {}

    fn draw(&self, ctx: karna::ContextRef, draw: &mut renderer::Draw) {
        let m = ctx.input.mouse_position();
        let win_size = ctx.window.size();

        draw.set_color(Color::Cyan);
        draw.line(0.0, m.y, win_size.width as f32, m.y);
        draw.line(m.x, 0.0, m.x, win_size.height as f32);

        draw.set_color(Color::Magenta);
        draw.circle_v(m, 5.0);

        draw.set_color(Color::White);
        draw.debug_text(&format!("Mouse x: {}", m.x), 10.0, 10.0);
        draw.debug_text(&format!("Mouse y: {}", m.y), 10.0, 30.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_title("Lines")
                .with_initial_scene(S),
        )
        .build()
        .run();
}

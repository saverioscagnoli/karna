use karna::{App, Color, Context, Scene};

struct S;

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render
            .load_image_from_bytes("bleh", include_bytes!("../assets/bleh.jpg"));

        ctx.render
            .load_image_from_bytes("4", include_bytes!("../assets/4.jpg"));
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.draw_image("bleh", [10, 10]);
        ctx.render.draw_image("4", [300, 300]);

        ctx.render.set_draw_color(Color::Red);

        ctx.render.fill_rect([200, 200], (50, 50));
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene("default", S)
        .run()
        .expect("Failed to run application");
}

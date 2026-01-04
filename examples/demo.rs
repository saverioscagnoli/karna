use karna::{
    AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder,
    assets::{Font, Image},
    input::KeyCode,
    math::Vector2,
    render::Color,
    utils::Handle,
};

struct Demo {
    cat: Handle<Image>,
    jetbrains_mono: Handle<Font>,
    pos: Vector2,
    color: Color,
}

impl Scene for Demo {
    fn load(&mut self, ctx: &mut Context) {
        self.cat = ctx
            .assets
            .load_image_bytes(include_bytes!("assets/cat.jpg").to_vec());

        self.jetbrains_mono = ctx
            .assets
            .load_font_bytes(include_bytes!("assets/jmono.ttf").to_vec(), 18);
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
        draw.set_color(Color::White);
        draw.debug_text("This is some nice debug text!!", 10.0, 10.0);
        draw.debug_text(format!("dt: {:.6}", ctx.time.delta()), 10.0, 30.0);

        draw.set_color(Color::Cyan);
        draw.text(
            self.jetbrains_mono,
            "This instead is JetBrains Mono!!",
            10.0,
            200.0,
        );

        draw.set_color(self.color);

        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);
        draw.image(self.cat, 400.0, 100.0);
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
                    cat: Handle::dummy(),
                    jetbrains_mono: Handle::dummy(),
                    pos: Vector2::new(10.0, 10.0),
                    color: Color::Red,
                }),
        )
        .build()
        .run();
}

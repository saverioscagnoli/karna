#![allow(unused)]

use karna::prelude::*;

const DEMO_SCENE: SceneId = SceneId::new_str("DEMO");

struct DemoScene {
    jbmono: Handle<Font>,
    dt_text: Text,
    jbmono_text: Text,
    pcb: Handle<Image>,
}

impl Scene for DemoScene {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        let jbmono = ctx.assets.load_font("assets/jbmono.ttf");

        Self {
            jbmono,
            dt_text: Text::default().with_style(TextStyle::default()),
            jbmono_text: Text::new("Hello world!")
                .with_style(TextStyle::default().with_font(jbmono)),
            pcb: ctx.assets.load_image("assets/pcb2.png"),
        }
    }

    fn fixed_update(&mut self, ctx: &mut UpdateContext) {}

    fn update(&mut self, ctx: &mut UpdateContext) {
        self.dt_text.set(format!("dt: {}", ctx.time.delta()));
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        draw.set_color(Color::RED);
        draw.rect(50.0, 50.0, 50.0, 50.0);

        draw.set_color(Color::CYAN);
        draw.circle(300.0, 200.0, 40.0);

        draw.set_color(Color::MAGENTA);
        draw.set_thickness(5.0);
        draw.circle_outline(300.0, 600.0, 35.0);

        draw.set_color(Color::YELLOW);
        draw.set_thickness(2.5);
        draw.line(20.0, 300.0, 400.0, 350.0);

        draw.set_color(Color::WHITE);
        draw.image(self.pcb, 600.0, 100.0);

        draw.with_layer(Layer::UI)
            .with_color(Color::WHITE)
            .with_thickness(3.0)
            .rect_outline(0.0, 400.0, 1280.0, 32.0);

        draw.set_color(Color::WHITE);
        draw.text(&self.dt_text, 10.0, 10.0);

        draw.set_color(Color::CYAN);
        draw.text(&self.jbmono_text, 10.0, 50.0);

        draw.text_style_mut().set_font(self.jbmono);
        draw.print("AAAAAAAAAAA", 400.0, 100.0);

        draw.text_style_mut().set_font(ctx.assets.debug_font());
        draw.set_color(Color::MAGENTA);
        draw.print("Debug font!\n(with a sexy new line!)", 400.0, 120.0);
    }
}

fn main() {
    _ = traccia::init(
        traccia::Config::default()
            .with_min_level(traccia::LevelFilter::Debug)
            .with_module_filter("cosmic_text", traccia::LevelFilter::Warn)
            .with_target(SdlTarget::default()),
    );

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Demo window")
                .with_size((1280, 720))
                .with_scene::<DemoScene>(DEMO_SCENE)
                .with_active_scene(DEMO_SCENE),
        )
        .with_root("karna/examples/")
        .build()
        .run();
}

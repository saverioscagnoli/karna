use karna::{
    AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder,
    assets::Font,
    input::KeyCode,
    render::{Color, Text, Transform3d},
    utils::Handle,
};
use logging::info;

#[derive(Default)]
struct TextDemo {
    font: Handle<Font>,
    text1: Handle<Text>,
    text2: Handle<Text>,
    color_timer: f32,
    logs_toggle: bool,
}

impl Scene for TextDemo {
    fn load(&mut self, ctx: &mut Context) {
        self.font = ctx
            .assets
            .load_font_bytes(include_bytes!("assets/jmono.ttf").to_vec(), 16);

        let mut text = Text::new(self.font).with_content("Hello world!");

        text.set_transform(Transform3d::default().with_position([100.0, 100.0, 0.0]));
        text.set_color(Color::Cyan);

        self.text1 = ctx.scene.add_text(text);

        let mut text =
            Text::new(ctx.assets.debug_font()).with_content("Retained text with debug font");

        text.set_position([300.0, 300.0, 0.0]);

        self.text2 = ctx.scene.add_text(text);
    }

    fn update(&mut self, ctx: &mut Context) {
        self.color_timer += ctx.time.delta();

        if let Some(font) = ctx.scene.get_text_mut(self.text1) {
            *font.rotation_z_mut() += 1.0 * ctx.time.delta();
        }

        if ctx.input.key_pressed(&KeyCode::KeyL) {
            self.logs_toggle = !self.logs_toggle;
        }
    }

    fn render(&mut self, _ctx: &RenderContext, draw: &mut Draw) {
        let r = self.color_timer.sin() * 127.0 + 128.0;
        let g = (self.color_timer + 2.0).sin() * 127.0 + 128.0;
        let b = (self.color_timer + 4.0).sin() * 127.0 + 128.0;

        draw.set_color(Color::rgb(r / 255.0, g / 255.0, b / 255.0));

        draw.text(
            self.font,
            "This is JetBrains Mono Text, but in immediate mode!\n(Rebuilds every frame, Should be used only for rapid-changing text)",
            10.0,
            10.0,
        );

        draw.debug_text("Press 'L' to toggle logs!", 10.0, 60.0);

        if self.logs_toggle {
            draw.debug_logs(10.0, 80.0);
        }
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Text demo")
                .with_resizable(false)
                .with_initial_scene(TextDemo::default()),
        )
        .build()
        .run();
}

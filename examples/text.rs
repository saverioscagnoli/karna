use karna::prelude::*;

const TEXT_DEMO_SCENE: SceneId = SceneId::new_str("text_demo");
const ATLAS_SCENE: SceneId = SceneId::new_str("atlas");

struct TextDemo {
    font: Handle<Font>,
    jbmono: Handle<Font>,
    style: TextStyle,
    str: String,
    f: f32,
    m: f32,
}

impl Scene for TextDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        ctx.window.start_text_input();

        Self {
            font: ctx.assets.system_font("Inter"),
            jbmono: ctx.assets.load_font("assets/jbmono.ttf"),
            style: TextStyle::default(),
            str: String::new(),
            f: 1.0,
            m: 1.0,
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        self.f += 0.1 * self.m;
        self.str.push_str(ctx.input.text());

        if self.f >= 100.0 {
            self.m = -1.0;
        } else if self.f <= 0.0 {
            self.m = 1.0;
        }

        if ctx.input.key_down(Key::Space) {
            self.style.font = Some(self.jbmono)
        }
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.print(&self.str, &self.style, 10.0, 10.0);
    }
}

struct Atlas;

impl Scene for Atlas {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn update(&mut self, ctx: &mut UpdateContext) {}

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        let atlas_page = draw.assets().atlas_page_image(0).unwrap();
        draw.textured(atlas_page, 0.0, 0.0, 1024.0, 1024.0, Color::WHITE);
    }
}

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Text Demo")
                .with_size((1280, 720))
                .with_scene::<TextDemo>(TEXT_DEMO_SCENE)
                .with_active_scene(TEXT_DEMO_SCENE),
        )
        .with_window(
            WindowBuilder::new()
                .with_title("Texture Atlas")
                .with_size((1024, 1024))
                .with_scene::<Atlas>(ATLAS_SCENE)
                .with_active_scene(ATLAS_SCENE),
        )
        .with_root("examples/")
        .build()
        .run();
}

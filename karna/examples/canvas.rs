use karna::prelude::*;

const CANVAS_SCENE: SceneId = SceneId::new_str("canvas");

const W: u32 = 320;
const H: u32 = 180;

struct CanvasDemo {
    canvas: Handle<Canvas>,
    witch: Sprite,
    t: f32,
}

impl Scene for CanvasDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        let idle = ctx.assets.load_image("assets/witch-idle.png");
        let sheet = ctx.assets.add_spritesheet(Spritesheet::strip(idle, 6, 8.0));

        Self {
            canvas: ctx.assets.add_canvas(W, H),
            witch: Sprite::new(sheet),
            t: 0.0,
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();

        self.t += dt;
        self.witch.update(dt);
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        draw.on_canvas(self.canvas, Color::rgba(0.1, 0.1, 0.15, 1.0), |d| {
            d.set_color(Color::rgba(0.3, 0.6, 0.9, 1.0));
            d.circle(W as f32 / 2.0, H as f32 / 2.0, 60.0);

            d.set_color(Color::WHITE);
            d.translate(W as f32 / 2.0, H as f32 / 2.0);
            d.rotate(self.t);
            d.sprite(&self.witch, -16.0, -24.0);
        });

        let size = ctx.window.size().cast::<f32>();

        draw.canvas_sized(self.canvas, 0.0, 0.0, size.w(), size.h());

        draw.set_color(Color::WHITE);
        draw.rect_outline(size.w() - 170.0, 10.0, 160.0, 90.0);
        draw.canvas_sized(self.canvas, size.w() - 170.0, 10.0, 160.0, 90.0);

        draw.print(format!("canvas {W}x{H} upscaled"), 10.0, 10.0);
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
        .with_root("karna/examples/")
        .with_window(
            WindowBuilder::default()
                .with_title("canvas demo")
                .with_size((1280, 720))
                .with_scene::<CanvasDemo>(CANVAS_SCENE)
                .with_active_scene(CANVAS_SCENE),
        )
        .build()
        .run();
}

use karna::prelude::*;

const SPRITE_SCENE: SceneId = SceneId::new_str("sprite");
const IDLE: Label = Label::new_str("idle");
const RUN: Label = Label::new_str("run");
const DEATH: Label = Label::new_str("death");

const SCALE: f32 = 6.0;

struct SpriteDemo {
    witch: Sprite,
    dead: bool,
    crowd: Vec<Sprite>,
}

impl Scene for SpriteDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);

        let idle = ctx.assets.load_image("assets/witch-idle.png");
        let run = ctx.assets.load_image("assets/witch-run.png");
        let death = ctx.assets.load_image("assets/witch-death.png");

        let sheet = ctx.assets.add_spritesheet(
            Spritesheet::new()
                .with_strip(IDLE, idle, 6, 8.0)
                .with_strip(RUN, run, 8, 12.0)
                .with_sequence(
                    DEATH,
                    death,
                    (0..12).map(|i| {
                        if i < 6 {
                            SpriteFrame::new([0, i * 48, 32, 48])
                        } else {
                            SpriteFrame::new([0, 288 + (i - 6) * 32, 32, 32]).with_offset(0, 16)
                        }
                    }),
                    10.0,
                )
                .once(),
        );

        let crowd_sheet = ctx.assets.add_spritesheet(Spritesheet::strip(idle, 6, 8.0));

        let crowd = (0..8)
            .map(|i| {
                let mut s = Sprite::new(crowd_sheet);
                s.update(i as f32 * 0.1);
                s
            })
            .collect();

        Self {
            witch: Sprite::new(sheet),
            dead: false,
            crowd,
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();

        if ctx.input.key_pressed(Key::Space) {
            self.dead = !self.dead;
        }

        if self.dead {
            self.witch.play(DEATH);
        } else if ctx.input.key_down(Key::A) || ctx.input.key_down(Key::D) {
            self.witch.play(RUN);
        } else {
            self.witch.play(IDLE);
        }

        self.witch.update(dt);

        for s in &mut self.crowd {
            s.update(dt);
        }
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        let size = ctx.window.size().cast::<f32>();
        let (w, h) = (32.0 * SCALE, 48.0 * SCALE);

        draw.sprite_sized(
            &self.witch,
            (size.w() - w) / 2.0,
            (size.h() - h) / 2.0,
            w,
            h,
        );

        for (i, s) in self.crowd.iter().enumerate() {
            draw.sprite(s, 20.0 + i as f32 * 40.0, size.h() - 68.0);
        }

        let frame = ctx
            .assets
            .try_spritesheet(self.witch.spritesheet())
            .and_then(|sheet| self.witch.frame(sheet));

        draw.print(format!("frame: {frame:?}"), 10.0, 10.0);
        draw.print("hold A / D to run, space to die / revive", 10.0, 30.0);
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
                .with_title("sprite demo")
                .with_size((1280, 720))
                .with_scene::<SpriteDemo>(SPRITE_SCENE)
                .with_active_scene(SPRITE_SCENE),
        )
        .build()
        .run();
}

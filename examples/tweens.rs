#![allow(unused)]
use std::f32::consts::TAU;
use std::time::Duration;

use karna::prelude::*;

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

const EASINGS: [(Easing, &str); 28] = [
    (Easing::Linear, "Linear"),
    (Easing::QuadIn, "QuadIn"),
    (Easing::QuadOut, "QuadOut"),
    (Easing::QuadInOut, "QuadInOut"),
    (Easing::CubicIn, "CubicIn"),
    (Easing::CubicOut, "CubicOut"),
    (Easing::CubicInOut, "CubicInOut"),
    (Easing::QuartIn, "QuartIn"),
    (Easing::QuartOut, "QuartOut"),
    (Easing::QuartInOut, "QuartInOut"),
    (Easing::QuintIn, "QuintIn"),
    (Easing::QuintOut, "QuintOut"),
    (Easing::QuintInOut, "QuintInOut"),
    (Easing::ExpoIn, "ExpoIn"),
    (Easing::ExpoOut, "ExpoOut"),
    (Easing::ExpoInOut, "ExpoInOut"),
    (Easing::CircIn, "CircIn"),
    (Easing::CircOut, "CircOut"),
    (Easing::CircInOut, "CircInOut"),
    (Easing::BackIn, "BackIn"),
    (Easing::BackOut, "BackOut"),
    (Easing::BackInOut, "BackInOut"),
    (Easing::ElasticIn, "ElasticIn"),
    (Easing::ElasticOut, "ElasticOut"),
    (Easing::ElasticInOut, "ElasticInOut"),
    (Easing::BounceIn, "BounceIn"),
    (Easing::BounceOut, "BounceOut"),
    (Easing::Custom(smoothstep), "Custom"),
];

fn started<T: Lerp>(mut t: Tween<T>) -> Tween<T> {
    t.start();
    t
}

struct TweenDemo {
    gallery: Vec<Tween<f32>>,

    loop_bars: Vec<(Tween<f32>, &'static str)>,

    ball_y: Tween<f32>,
    pulse_a: Tween<f32>,
    spin: Tween<f32>,
    pop: Tween<f32>,
    panel: Tween<f32>,
    wander_x: Tween<f32>,
    wander_y: Tween<f32>,
    counter: Tween<u8>,
}

impl TweenDemo {
    fn new() -> Self {
        let gallery = EASINGS
            .iter()
            .map(|&(easing, _)| {
                started(
                    Tween::new(0.0f32, 1.0, easing, Duration::from_secs_f32(1.6))
                        .with_loop_mode(LoopMode::Yoyo),
                )
            })
            .collect();

        let bar = |mode: LoopMode, name: &'static str| {
            (
                started(
                    Tween::new(0.0f32, 1.0, Easing::Linear, Duration::from_secs_f32(2.0))
                        .with_loop_mode(mode),
                ),
                name,
            )
        };

        let loop_bars = vec![
            bar(LoopMode::Once, "Once"),
            bar(LoopMode::Repeat, "Repeat"),
            bar(LoopMode::RepeatN(2), "RepeatN(2)"),
            bar(LoopMode::Yoyo, "Yoyo"),
            bar(LoopMode::YoyoN(2), "YoyoN(2)"),
        ];

        let mut pop = Tween::new(0.4f32, 1.0, Easing::BackOut, Duration::from_secs_f32(0.5));
        pop.on_complete(|_| {
            karna::logging::info!("pop finished (on_complete fired)");
        });

        let panel = Tween::new(0.0f32, 1.0, Easing::QuadOut, Duration::from_secs_f32(0.4));

        Self {
            gallery,
            loop_bars,
            ball_y: started(
                Tween::new(
                    230.0f32,
                    430.0,
                    Easing::BounceOut,
                    Duration::from_secs_f32(1.6),
                )
                .with_loop_mode(LoopMode::Repeat),
            ),
            pulse_a: started(
                Tween::new(
                    0.15f32,
                    1.0,
                    Easing::QuadInOut,
                    Duration::from_secs_f32(0.8),
                )
                .with_loop_mode(LoopMode::Yoyo),
            ),
            spin: started(Tween::new(
                0.0f32,
                TAU,
                Easing::ElasticOut,
                Duration::from_secs_f32(1.8),
            )),
            pop: started(pop),
            panel,
            wander_x: started(
                Tween::new(
                    950.0f32,
                    1200.0,
                    Easing::QuadInOut,
                    Duration::from_secs_f32(2.3),
                )
                .with_loop_mode(LoopMode::Yoyo),
            ),
            wander_y: started(
                Tween::new(
                    470.0f32,
                    640.0,
                    Easing::QuadInOut,
                    Duration::from_secs_f32(3.1),
                )
                .with_loop_mode(LoopMode::Yoyo),
            ),
            counter: started(
                Tween::new(0u8, 200, Easing::BackInOut, Duration::from_secs_f32(2.5))
                    .with_loop_mode(LoopMode::Yoyo),
            ),
        }
    }
}

impl Scene for TweenDemo {
    fn load(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        let dt = ctx.time.delta();

        for t in &mut self.gallery {
            t.update(dt);
        }
        for (t, _) in &mut self.loop_bars {
            t.update(dt);
        }
        self.ball_y.update(dt);
        self.pulse_a.update(dt);
        self.spin.update(dt);
        self.pop.update(dt);
        self.panel.update(dt);
        self.wander_x.update(dt);
        self.wander_y.update(dt);
        self.counter.update(dt);

        if ctx.input.key_pressed(Keycode::Space) {
            self.pop.reset();
            self.pop.start();
        }

        if ctx.input.key_pressed(Keycode::KeyR) {
            self.spin.reset();
            self.spin.start();
        }

        if ctx.input.key_pressed(Keycode::Tab) {
            self.panel.toggle_direction();
        }
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        draw.set_clear_color(Color::rgb(0.08, 0.09, 0.11));

        draw.set_color(Color::White);
        draw.debug_text("easing gallery (yoyo)", 30.0, 20.0);

        let (track_x, track_w) = (150.0, 220.0);

        for (i, t) in self.gallery.iter().enumerate() {
            let y = 52.0 + i as f32 * 22.0;

            draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.6));
            draw.debug_text(EASINGS[i].1, 30.0, y - 6.0);

            draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.15));
            draw.set_line_width(2.0);
            draw.line(track_x, y, track_x + track_w, y);

            draw.set_color(Color::Cyan);
            draw.circle(track_x + t.value() * track_w, y, 6.0);
        }

        draw.set_line_width(1.0);

        draw.set_color(Color::White);
        draw.debug_text("loop modes (2s linear)", 450.0, 20.0);

        for (i, (t, name)) in self.loop_bars.iter().enumerate() {
            let y = 50.0 + i as f32 * 30.0;

            draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.6));
            draw.debug_text(name, 450.0, y - 4.0);

            draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.12));
            draw.rect(560.0, y - 8.0, 300.0, 16.0);
            draw.set_color(if t.is_complete() {
                Color::Orange
            } else {
                Color::Cyan
            });
            draw.rect(560.0, y - 8.0, 300.0 * t.value(), 16.0);
        }

        draw.set_color(Color::White);
        draw.debug_text("BounceOut", 460.0, 230.0);
        draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.15));
        draw.line(430.0, 445.0, 560.0, 445.0); // floor
        draw.set_color(Color::Red);
        draw.circle(495.0, self.ball_y.value(), 14.0);

        draw.set_color(Color::White);
        draw.debug_text("ElasticOut [R]", 610.0, 230.0);
        draw.push_state();
        draw.translate(660.0, 330.0);
        draw.rotate(self.spin.value());
        draw.set_color(Color::Cyan);
        draw.rect(-28.0, -28.0, 56.0, 56.0);
        draw.pop_state();

        draw.set_color(Color::White);
        draw.debug_text("alpha yoyo", 760.0, 230.0);
        draw.set_color(Color::rgba(1.0, 0.3, 0.8, self.pulse_a.value()));
        draw.circle(810.0, 330.0, 32.0);

        draw.set_color(Color::White);
        draw.debug_text("BackOut pop [Space]", 890.0, 230.0);
        let s = self.pop.value();
        draw.push_state();
        draw.translate(960.0, 330.0);
        draw.scale(s, s);
        draw.set_color(Color::Orange);
        draw.rounded_rect(-30.0, -30.0, 60.0, 60.0, 12.0);
        draw.pop_state();

        draw.set_color(Color::White);
        draw.debug_text(
            &format!("u8 BackInOut yoyo: {:>3}", self.counter.value()),
            610.0,
            470.0,
        );
        draw.set_color(Color::Cyan);
        draw.rect(610.0, 490.0, self.counter.value() as f32, 10.0);

        draw.set_color(Color::White);
        draw.debug_text("2 tweens, 2 periods", 950.0, 440.0);
        draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.12));
        draw.rect_outline(940.0, 460.0, 280.0, 200.0, 1.0);
        draw.set_color(Color::Magenta);
        draw.circle(self.wander_x.value(), self.wander_y.value(), 8.0);

        let v = self.panel.value();
        let panel_y = 720.0 - v * 160.0;

        draw.set_color(Color::rgba(0.15, 0.17, 0.22, 0.95));
        draw.rounded_rect(430.0, panel_y, 380.0, 150.0, 14.0);
        draw.set_color(Color::Cyan);
        draw.rounded_rect_outline(430.0, panel_y, 380.0, 150.0, 14.0, 2.0);
        draw.set_color(Color::White);
        draw.debug_text("QuadOut slide — spam [Tab]:", 450.0, panel_y + 20.0);
        draw.debug_text("reversal is smooth mid-flight,", 450.0, panel_y + 44.0);
        draw.debug_text("no snapping (toggle_direction)", 450.0, panel_y + 68.0);

        draw.set_layer(Layer::Ui);
        draw.set_color(Color::White);
        draw.debug_text(
            &format!(
                "fps {}   [Space] pop  [R] spin  [Tab] panel",
                ctx.time.fps()
            ),
            30.0,
            694.0,
        );
    }
}

fn main() {
    karna::logging::init(
        karna::logging::Config {
            min_level: karna::logging::LevelFilter::Debug,
            ..Default::default()
        }
        .hide_wgpu(true),
    )
    .expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_scene("tweens", TweenDemo::new())
                .with_active_scene("tweens"),
        )
        .build()
        .run();
}

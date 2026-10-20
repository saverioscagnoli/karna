use core::f32::consts::TAU;
use core::time::Duration;

use karna::prelude::*;

const TWEEN_SCENE: SceneId = SceneId::new_str("tween");

const ROWS: usize = 18;
const ROW_H: f32 = 34.0;
const COL_W: f32 = 410.0;
const LABEL_W: f32 = 120.0;
const TRACK_W: f32 = 250.0;
const TOP: f32 = 60.0;

fn row(i: usize) -> (f32, f32) {
    (
        20.0 + (i / ROWS) as f32 * COL_W,
        TOP + (i % ROWS) as f32 * ROW_H,
    )
}

fn row_box(i: usize) -> Aabb2<f32> {
    let (x, y) = row(i);
    Aabb2::new(x, y - ROW_H / 2.0, LABEL_W + TRACK_W + 12.0, ROW_H)
}

fn entries() -> Vec<(String, Ease)> {
    Ease::ALL.iter().map(|e| (format!("{e:?}"), *e)).collect()
}

fn panel() -> Aabb2<f32> {
    Aabb2::new(850.0, 60.0, 410.0, 600.0)
}

const PALETTE: [Color; 5] = [
    Color::rgb(0.95, 0.35, 0.35),
    Color::rgb(0.35, 0.8, 0.95),
    Color::rgb(0.55, 0.9, 0.45),
    Color::rgb(0.95, 0.8, 0.3),
    Color::rgb(0.75, 0.5, 0.95),
];

struct TweenDemo {
    entries: Vec<(String, Ease)>,
    tracks: Vec<Tween<f32>>,
    ease: usize,
    position: Tween<Vector2<f32>>,
    color: Tween<Color>,
    spin: Tween<f32>,
    pulse: Tween<Size<f32>>,
    palette: usize,
}

impl TweenDemo {
    fn ease(&self) -> Ease {
        self.entries[self.ease].1
    }
}

impl Scene for TweenDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        if let Some(monitor) = ctx.monitors.primary() {
            ctx.time
                .set_target_fps(monitor.refresh_rate().round() as u32);
        }

        let entries = entries();
        let tracks = entries
            .iter()
            .map(|(_, e)| {
                Tween::new(0.0, 1.0, Duration::from_millis(1600))
                    .with_ease(*e)
                    .with_repeat(Repeat::PingPong)
            })
            .collect();

        let center = panel().center();
        let ease = entries
            .iter()
            .position(|(_, e)| *e == Ease::BackOut)
            .unwrap_or(0);

        Self {
            entries,
            tracks,
            ease,
            position: Tween::new(center, center, Duration::from_millis(700))
                .with_ease(Ease::BackOut),
            color: Tween::new(PALETTE[0], PALETTE[0], Duration::from_millis(500)),
            spin: Tween::new(0.0, 0.0, Duration::from_millis(900)).with_ease(Ease::BackOut),
            pulse: Tween::new(
                Size::new(70.0, 70.0),
                Size::new(90.0, 90.0),
                Duration::from_millis(800),
            )
            .with_ease(Ease::SineInOut)
            .with_repeat(Repeat::PingPong),
            palette: 0,
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();

        for t in &mut self.tracks {
            t.update(dt);
        }

        let count = self.entries.len();

        if ctx.input.key_pressed(Key::Right) {
            self.ease = (self.ease + 1) % count;
        }
        if ctx.input.key_pressed(Key::Left) {
            self.ease = (self.ease + count - 1) % count;
        }

        let mouse = ctx.window.mouse_position();
        let clicked = ctx.input.mouse_pressed(MouseButton::Left);

        if clicked && let Some(i) = (0..count).find(|i| row_box(*i).contains(mouse)) {
            self.ease = i;
        }

        let ease = self.ease();
        self.position.set_ease(ease);
        self.spin.set_ease(ease);

        if clicked && panel().contains(mouse) {
            self.position.retarget(mouse);

            self.palette = (self.palette + 1) % PALETTE.len();
            self.color.retarget(PALETTE[self.palette]);

            let angle = self.spin.value();
            self.spin = self.spin.with_from(angle % TAU).with_to(angle % TAU + TAU);
            self.spin.reset();
        }

        self.position.update(dt);
        self.color.update(dt);
        self.spin.update(dt);
        self.pulse.update(dt);
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        let mouse = ctx.window.mouse_position();

        for (i, ((name, _), tween)) in self.entries.iter().zip(&self.tracks).enumerate() {
            let (x, y) = row(i);
            let track_x = x + LABEL_W;
            let selected = i == self.ease;

            if selected || row_box(i).contains(mouse) {
                let b = row_box(i);
                draw.set_color(Color::rgba(
                    1.0,
                    1.0,
                    1.0,
                    if selected { 0.12 } else { 0.06 },
                ));
                draw.rect_v(b.min, b.size());
            }

            draw.set_color(if selected {
                Color::YELLOW
            } else {
                Color::WHITE
            });
            draw.print(name, x + 4.0, y - 8.0);

            draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.2));
            draw.set_thickness(2.0);
            draw.line(track_x, y, track_x + TRACK_W, y);

            draw.set_color(if selected { Color::YELLOW } else { Color::CYAN });
            draw.circle(track_x + tween.value() * TRACK_W, y, 6.0);
        }

        draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.25));
        draw.set_thickness(2.0);
        draw.rect_outline_v(panel().min, panel().size());

        let pos = self.position.value();
        let size = self.pulse.value();

        draw.scoped(|d| {
            d.translate(pos.x, pos.y);
            d.rotate(self.spin.value());
            d.set_color(self.color.value());
            d.rect(-size.w() / 2.0, -size.h() / 2.0, size.w(), size.h());
        });

        draw.set_color(Color::WHITE);
        draw.print("easings taken from easings.net, ping-ponging", 20.0, 16.0);
        draw.print(
            "click a row or use left/right to pick, click the box to move",
            panel().min.x,
            panel().max.y + 8.0,
        );
        draw.print(
            format!("ease: {:?}", self.ease()),
            panel().min.x,
            panel().max.y + 28.0,
        );
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
                .with_title("tween demo")
                .with_size((1280, 720))
                .with_scene::<TweenDemo>(TWEEN_SCENE)
                .with_active_scene(TWEEN_SCENE),
        )
        .build()
        .run();
}

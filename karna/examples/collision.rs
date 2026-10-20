use karna::math::Aabb2;
use karna::math::Vector2;
use karna::prelude::*;

const COLLISION_SCENE: SceneId = SceneId::new_str("collision");

const SPEED: f32 = 320.0;
const GRAVITY: f32 = 1800.0;
const JUMP: f32 = 720.0;
const MAX_FALL: f32 = 1200.0;

struct CollisionDemo {
    player: Aabb2<f32>,
    velocity: Vector2<f32>,
    on_ground: bool,
    walls: Vec<Aabb2<f32>>,
    probe: Aabb2<f32>,
}

impl CollisionDemo {
    fn resolve(&mut self) {
        self.on_ground = false;

        for wall in &self.walls {
            let Some(push) = self.player.penetration(wall) else {
                continue;
            };

            self.player.translate(push);

            if push.y < 0.0 && self.velocity.y > 0.0 {
                self.velocity.y = 0.0;
                self.on_ground = true;
            }

            if push.y > 0.0 && self.velocity.y < 0.0 {
                self.velocity.y = 0.0;
            }

            if push.x != 0.0 {
                self.velocity.x = 0.0;
            }
        }
    }
}

impl Scene for CollisionDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);

        let walls = vec![
            Aabb2::new(0.0, 660.0, 1280.0, 60.0),
            Aabb2::new(0.0, 0.0, 20.0, 720.0),
            Aabb2::new(1260.0, 0.0, 20.0, 720.0),
            Aabb2::new(200.0, 520.0, 240.0, 24.0),
            Aabb2::new(520.0, 400.0, 200.0, 24.0),
            Aabb2::new(820.0, 280.0, 260.0, 24.0),
            Aabb2::new(640.0, 560.0, 60.0, 100.0),
            Aabb2::new(980.0, 480.0, 120.0, 24.0),
        ];

        Self {
            player: Aabb2::new(80.0, 500.0, 32.0, 48.0),
            velocity: Vector2::zero(),
            on_ground: false,
            walls,
            probe: Aabb2::new(0.0, 0.0, 80.0, 80.0),
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta().min(1.0 / 30.0);

        let mut dir = 0.0;

        if ctx.input.key_down(Key::A) {
            dir -= 1.0;
        }
        if ctx.input.key_down(Key::D) {
            dir += 1.0;
        }

        self.velocity.x = dir * SPEED;
        self.velocity.y = (self.velocity.y + GRAVITY * dt).min(MAX_FALL);

        if self.on_ground && (ctx.input.key_pressed(Key::Space) || ctx.input.key_pressed(Key::W)) {
            self.velocity.y = -JUMP;
        }

        if ctx.input.key_pressed(Key::R) {
            self.player = Aabb2::new(80.0, 500.0, 32.0, 48.0);
            self.velocity = Vector2::zero();
        }

        self.player.translate(self.velocity * dt);
        self.resolve();

        let mouse = ctx.window.mouse_position();
        let half = self.probe.extent() / 2.0;
        self.probe = Aabb2::from_center(mouse, half);
    }

    fn draw(&mut self, _ctx: &mut DrawContext, draw: &mut Draw) {
        draw.set_color(Color::GRAY);
        for wall in &self.walls {
            draw.rect_v(wall.min, wall.size());
        }

        let touching = self.walls.iter().any(|w| w.intersects(&self.probe))
            || self.player.intersects(&self.probe);

        draw.set_color(if touching { Color::RED } else { Color::GREEN });
        draw.set_thickness(2.0);
        draw.rect_outline_v(self.probe.min, self.probe.size());

        draw.set_color(Color::rgba(1.0, 0.65, 0.0, 0.6));
        for overlap in self
            .walls
            .iter()
            .chain(core::iter::once(&self.player))
            .filter_map(|w| w.intersection(&self.probe))
        {
            draw.rect_v(overlap.min, overlap.size());
        }

        draw.set_color(if self.on_ground { Color::CYAN } else { Color::YELLOW });
        draw.rect_v(self.player.min, self.player.size());

        draw.set_color(Color::WHITE);
        draw.print("A/D move, Space/W jump, R reset, mouse moves the probe", 30.0, 20.0);
        draw.print(
            format!(
                "on ground: {}  velocity: ({:.0}, {:.0})",
                self.on_ground, self.velocity.x, self.velocity.y
            ),
            30.0,
            40.0,
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
                .with_title("collision demo")
                .with_size((1280, 720))
                .with_scene::<CollisionDemo>(COLLISION_SCENE)
                .with_active_scene(COLLISION_SCENE),
        )
        .build()
        .run();
}

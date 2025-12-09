use karna::{
    AppBuilder, Context, Scene, WindowBuilder,
    input::KeyCode,
    label,
    math::Vector2,
    render::{Color, Material, Mesh, MeshGeometry, TextureKind, Transform},
};
use renderer::Text;

pub struct S {
    rect: Mesh,
    vel: Vector2,
    circle: Mesh,
    debug_text: Text,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::Black);
        ctx.audio
            .load_from_bytes(label!("mammamia"), include_bytes!("assets/mamma-mia.mp3"));

        ctx.render
            .load_texture(label!("cat"), include_bytes!("assets/cat.jpg"));

        ctx.render.load_font(
            label!("jetbrains mono"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf"),
            16,
        );
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.key_pressed(&KeyCode::F11) {
            if ctx.window.is_fullscreen() {
                ctx.window.set_windowed();
            } else {
                ctx.window.set_fullscreen();
            }
        }

        let vel = 250.0;

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y = -vel;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x = -vel;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y = vel;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x = vel;
        }

        self.rect.position += self.vel * ctx.time.delta();
        self.vel *= 0.9;

        if ctx.input.key_pressed(&KeyCode::Space) {
            ctx.audio.play(label!("mammamia"));
        }

        self.debug_text.content = format!("fps: {}\ndt: {}", ctx.time.fps(), ctx.time.delta());
    }

    fn render(&mut self, ctx: &mut Context) {
        self.rect.render(&mut ctx.render);
        ctx.render.draw_mesh(&self.circle);

        self.debug_text.render(&ctx.gpu, &mut ctx.render);
    }
}

pub struct S2 {
    circle: Mesh,
}

impl Scene for S2 {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::Black);
    }

    fn update(&mut self, ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.draw_mesh(&self.circle);
        ctx.render.draw_texture_atlas([0.0, 0.0]);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new().with_initial_scene(Box::new(S {
                rect: Mesh {
                    geometry: MeshGeometry::rect(),
                    material: Material {
                        texture: Some(TextureKind::Full(label!("cat"))),
                        color: None,
                    },
                    transform: Transform::default()
                        .with_position([10.0, 10.0])
                        .with_scale([50.0, 50.0]),
                },
                vel: Vector2::zeros(),
                circle: Mesh {
                    geometry: MeshGeometry::circle(50.0, 32),
                    material: Material {
                        texture: None,
                        color: Some(Color::Cyan),
                    },
                    transform: Transform::default().with_position([200.0, 200.0]),
                },
                debug_text: Text::new(label!("jetbrains mono"), ""),
            })),
        )
        .with_window(
            WindowBuilder::new()
                .with_size((1024, 1024))
                .with_initial_scene(Box::new(S2 {
                    circle: Mesh {
                        geometry: MeshGeometry::circle(50.0, 32),
                        material: Material {
                            texture: None,
                            color: Some(Color::Cyan),
                        },
                        transform: Transform::default().with_position([250.0, 250.0]),
                    },
                })),
        )
        .build()
        .run();
}

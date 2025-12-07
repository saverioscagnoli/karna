use karna::{
    App, AppBuilder, Context, FullscreenMode, Scene, WindowBuilder, input::KeyCode, render::Color,
    render::Mesh,
};
use renderer::{MeshGeometry, Transform};

pub struct S {
    rect: Mesh,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::Gray);
    }

    fn update(&mut self, ctx: &mut Context) {
        //println!("fps {}, dt {}", ctx.time.fps(), ctx.time.delta());
        if ctx.input.key_pressed(&KeyCode::Space) {
            if ctx.window.is_fullscreen() {
                ctx.window.set_windowed();
            } else {
                ctx.window.set_fullscreen();
            }
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        self.rect.render(&mut ctx.render);
    }
}

pub struct S2 {
    circle: Mesh,
}

impl Scene for S2 {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::White);
    }

    fn update(&mut self, ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.draw_mesh(&self.circle);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new().with_initial_scene(Box::new(S {
                rect: Mesh {
                    geometry: MeshGeometry::rect(),
                    color: Color::Red,
                    transform: Transform::default()
                        .with_position([10.0, 10.0])
                        .with_scale([50.0, 50.0]),
                },
            })),
        )
        .with_window(WindowBuilder::new().with_initial_scene(Box::new(S2 {
            circle: Mesh {
                geometry: MeshGeometry::circle(50.0, 32),
                color: Color::Cyan,
                transform: Transform::default().with_position([250.0, 250.0]),
            },
        })))
        .build()
        .run();
}

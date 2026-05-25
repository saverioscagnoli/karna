mod bomberman;
mod consts;
mod level;

use karna::App;
use karna::Scene;
use karna::WindowBuilder;
use utils::Lazy;

use crate::bomberman::Bomberman;
use crate::consts::SCREEN_HEIGHT;
use crate::consts::SCREEN_WIDTH;
use crate::level::Level;

pub struct S {
    bomberman: Lazy<Bomberman>,
    level: Lazy<Level>,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        // Create the level after we loaded other assets, because this consumes `ctx`.
        self.level
            .set(Level::new(include_str!("levels/1.lvl"), ctx).expect("invalid level"));
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {
        self.level.update(ctx);
    }

    fn draw(&self, ctx: karna::ContextRef, draw: &mut renderer::Draw) {
        self.level.render(draw);
    }
}

struct D;

impl Scene for D {
    fn load(&mut self, ctx: karna::ContextRefMut) {}

    fn update(&mut self, ctx: karna::ContextRefMut) {}

    fn draw(&self, ctx: karna::ContextRef, draw: &mut renderer::Draw) {}
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::default()
                .with_title("Sprite demo")
                .with_size((SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))
                .with_initial_scene(S {
                    bomberman: Lazy::new(),
                    level: Lazy::new(),
                }),
        )
        .build()
        .run()
}

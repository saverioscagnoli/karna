use karna::prelude::*;

const RED: SceneId = SceneId::new_str("red");
const BLUE: SceneId = SceneId::new_str("blue");

struct Red;

impl Scene for Red {
    fn load(_ctx: &mut LoadContext) -> Self {
        Self
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        if ctx.input.key_pressed(Key::Space) {
            ctx.scene.change(BLUE);
        }
    }

    fn draw(&mut self, _ctx: &mut DrawContext, draw: &mut Draw) {
        draw.set_color(Color::RED);
        draw.print("red scene, press space", 10.0, 10.0);
    }
}

struct Blue;

impl Scene for Blue {
    fn load(_ctx: &mut LoadContext) -> Self {
        Self
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        if ctx.input.key_pressed(Key::Space) {
            ctx.scene.change(RED);
        }
    }

    fn draw(&mut self, _ctx: &mut DrawContext, draw: &mut Draw) {
        draw.set_color(Color::BLUE);
        draw.print("blue scene, press space", 10.0, 10.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::default()
                .with_title("scenes")
                .with_size((800, 600))
                .with_scene::<Red>(RED)
                .with_scene::<Blue>(BLUE)
                .with_active_scene(RED),
        )
        .build()
        .run();
}

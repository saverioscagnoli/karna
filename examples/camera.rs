use karna::{App, Scene, WindowBuilder, input::KeyCode, math::Lerp};
use math::Vector2;
use renderer::{Color, Geometry, Material, Mesh, Transform};

struct CameraDemo {
    player: Mesh,
    corner_rects: Vec<Mesh>,
}

impl Scene for CameraDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        let size = ctx.window.size().to_f32();

        for i in 0..4 {
            let x = if i % 2 == 0 { 0.0 } else { size.width - 50.0 };
            let y = if i < 2 { 0.0 } else { size.height - 50.0 };

            let mesh = Mesh::new(
                Geometry::rect(50.0, 50.0),
                Material::new_color(Color::random()),
                Transform::default().with_position(Vector2::new(x, y)),
            );

            self.corner_rects.push(mesh);
            self.player
                .set_position(size.centered_tl(&(50.0, 50.0).into()));

            ctx.time.set_target_fps(120);
            ctx.render.set_clear_color(Color::Black);
        }
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        // Increased velocity for faster player movement
        let vel = 400.0 * ctx.time.delta();

        if ctx.input.key_held(&KeyCode::KeyW) {
            *self.player.position_y_mut() -= vel;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            *self.player.position_y_mut() += vel;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            *self.player.position_x_mut() -= vel;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            *self.player.position_x_mut() += vel;
        }

        let player_center = *self.player.position() + Vector2::new(25.0, 25.0);
        let screen_center_offset = ctx.window.size().to_f32().center();
        let target_camera_pos = player_center - screen_center_offset;

        let current_camera_pos = ctx.render.camera.position();
        let new_pos = current_camera_pos.lerp(&target_camera_pos, 0.05);
        ctx.render.camera.set_position(new_pos);
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_mesh(&self.player);

        for mesh in &self.corner_rects {
            ctx.render.draw_mesh(mesh);
        }

        // Debug text examples - text objects are cached and only rebuilt when content changes
        let fps = ctx.time.fps();
        ctx.render
            .draw_ui_debug_text(format!("FPS: {:.0}", fps), Vector2::new(10.0, 10.0));

        let pos = self.player.position();
        ctx.render.draw_ui_debug_text(
            format!("Player: ({:.1}, {:.1})", pos.x, pos.y),
            Vector2::new(10.0, 30.0),
        );

        let cam_pos = ctx.render.camera.position();

        ctx.render.draw_ui_debug_text(
            format!("Camera: ({:.1}, {:.1})", cam_pos.x, cam_pos.y),
            Vector2::new(10.0, 50.0),
        );
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("camera movement demo")
                .with_resizable(false)
                .with_initial_scene(CameraDemo {
                    player: Mesh::new(
                        Geometry::rect(50.0, 50.0),
                        Material::new_color(Color::Cyan),
                        Transform::default(),
                    ),
                    corner_rects: Vec::new(),
                }),
        )
        .build()
        .run();
}

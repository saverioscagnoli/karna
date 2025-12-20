use karna::{App, Scene, WindowBuilder, input::KeyCode, math::Lerp};
use math::Vector2;
use renderer::{Color, Geometry, Material, Mesh, Text, Transform};
use utils::label;

struct UiCameraDemo {
    player: Mesh,
    corner_rects: Vec<Mesh>,

    // UI elements - these will be positioned in screen space
    ui_text: Text,
    fps_text: Text,
    position_text: Text,
    camera_text: Text,
}

impl Scene for UiCameraDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        let size = ctx.window.size().to_f32();

        // Create corner rectangles that move with the world camera
        for i in 0..4 {
            let x = if i % 2 == 0 { 0.0 } else { size.width - 50.0 };
            let y = if i < 2 { 0.0 } else { size.height - 50.0 };

            let mesh = Mesh::new(
                Geometry::rect(50.0, 50.0),
                Material::new_color(Color::random()),
                Transform::default().with_position(Vector2::new(x, y)),
            );

            self.corner_rects.push(mesh);
        }

        // Position player at center
        self.player
            .set_position(size.centered_tl(&(50.0, 50.0).into()));

        // Set up UI text elements with fixed screen positions
        self.ui_text
            .set_content("UI Camera Demo - WASD to move player");
        self.ui_text.set_position(Vector2::new(10.0, 10.0));
        self.ui_text.set_color(Color::Yellow);

        self.fps_text.set_position(Vector2::new(10.0, 35.0));
        self.fps_text.set_color(Color::Cyan);

        self.position_text.set_position(Vector2::new(10.0, 60.0));
        self.position_text.set_color(Color::Green);

        self.camera_text.set_position(Vector2::new(10.0, 85.0));
        self.camera_text.set_color(Color::Magenta);

        ctx.time.set_target_fps(120);
        ctx.render.set_clear_color(Color::from_hex("#1a1a2e"));
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let vel = 400.0 * ctx.time.delta();

        // Move player with WASD
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

        // Camera smoothly follows player (this is the world camera)
        let player_center = *self.player.position() + Vector2::new(25.0, 25.0);
        let screen_center_offset = ctx.window.size().to_f32().center();
        let target_camera_pos = player_center - screen_center_offset;

        let current_camera_pos = ctx.render.camera.position();
        let new_pos = current_camera_pos.lerp(&target_camera_pos, 0.08);
        ctx.render.camera.set_position(new_pos);

        // Update UI text content
        self.fps_text
            .set_content(format!("FPS: {:.0}", ctx.time.fps()));

        let pos = self.player.position();
        self.position_text
            .set_content(format!("Player: ({:.1}, {:.1})", pos.x, pos.y));

        let cam_pos = ctx.render.camera.position();
        self.camera_text
            .set_content(format!("Camera: ({:.1}, {:.1})", cam_pos.x, cam_pos.y));
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        // Draw world objects (affected by main camera)
        ctx.render.draw_mesh(&self.player);
        for mesh in &self.corner_rects {
            ctx.render.draw_mesh(mesh);
        }

        // WORKAROUND: To keep UI text fixed on screen while the world camera moves,
        // we need to offset the text position by the camera position.
        // This effectively cancels out the camera transformation.

        let cam_offset = ctx.render.camera.position();

        // Save original positions
        let ui_pos = *self.ui_text.position();
        let fps_pos = *self.fps_text.position();
        let position_pos = *self.position_text.position();
        let camera_pos = *self.camera_text.position();

        // Offset by camera position to keep text fixed on screen
        self.ui_text.set_position(ui_pos + cam_offset);
        self.fps_text.set_position(fps_pos + cam_offset);
        self.position_text.set_position(position_pos + cam_offset);
        self.camera_text.set_position(camera_pos + cam_offset);

        // Draw UI text
        ctx.render.draw_text(&mut self.ui_text);
        ctx.render.draw_text(&mut self.fps_text);
        ctx.render.draw_text(&mut self.position_text);
        ctx.render.draw_text(&mut self.camera_text);

        // Restore original positions for next frame
        self.ui_text.set_position(ui_pos);
        self.fps_text.set_position(fps_pos);
        self.position_text.set_position(position_pos);
        self.camera_text.set_position(camera_pos);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("UI Camera Demo - Text stays fixed while camera moves")
                .with_resizable(false)
                .with_initial_scene(UiCameraDemo {
                    player: Mesh::new(
                        Geometry::rect(50.0, 50.0),
                        Material::new_color(Color::Cyan),
                        Transform::default(),
                    ),
                    corner_rects: Vec::new(),
                    ui_text: Text::new(label!("debug")),
                    fps_text: Text::new(label!("debug")),
                    position_text: Text::new(label!("debug")),
                    camera_text: Text::new(label!("debug")),
                }),
        )
        .build()
        .run();
}

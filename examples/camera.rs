use std::time::Duration;

use karna::{App, Scene, WindowBuilder, input::KeyCode, math::Lerp};
use math::{Size, Vector2, Vector3};
use renderer::{Color, Geometry, Layer, Material, Mesh, MeshHandle, Transform};

struct CameraDemo {
    player: MeshHandle,
    corner_rects: Vec<MeshHandle>,
}

impl Scene for CameraDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.time.uncap_fps();
        ctx.render.set_active_layer(Layer::World);

        let mut player = Mesh::new(
            Geometry::rect((50.0, 50.0)),
            Material::new_color(Color::Cyan),
            Transform::default(),
        );

        let size = ctx.window.size().to_f32();
        let centered = size.centered_tl(&Size::new(50.0, 50.0));
        let position = Vector3::new(centered.x, centered.y, 0.0);

        player.set_position(position);

        self.player = ctx.render.add_mesh(player);

        for i in 0..4 {
            let x = if i % 2 == 0 { 0.0 } else { size.width - 50.0 };
            let y = if i < 2 { 0.0 } else { size.height - 50.0 };

            let mesh = Mesh::new(
                Geometry::rect((50.0, 50.0)),
                Material::new_color(Color::random()),
                Transform::default().with_position(Vector3::new(x, y, 0.0)),
            );

            let mesh_id = ctx.render.add_mesh(mesh);

            self.corner_rects.push(mesh_id);
        }

        ctx.render.set_clear_color(Color::Black);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_active_layer(Layer::World);
        let vel = 400.0 * ctx.time.delta();

        let player_position = {
            let player = ctx.render.get_mesh_mut(self.player);
            if ctx.input.key_held(&KeyCode::KeyW) {
                *player.position_y_mut() -= vel;
            }

            if ctx.input.key_held(&KeyCode::KeyS) {
                *player.position_y_mut() += vel;
            }

            if ctx.input.key_held(&KeyCode::KeyA) {
                *player.position_x_mut() -= vel;
            }

            if ctx.input.key_held(&KeyCode::KeyD) {
                *player.position_x_mut() += vel;
            }

            *player.position()
        };

        let camera = ctx.render.camera_mut();

        if ctx.input.key_pressed(&KeyCode::Space) {
            camera.shake(8.0, Duration::from_secs(2));
        }

        let player_center = player_position + Vector3::new(25.0, 25.0, 0.0);
        let screen_center_offset = ctx.window.size().to_f32().center();
        let target_camera_pos = Vector2::new(
            player_center.x - screen_center_offset.x,
            player_center.y - screen_center_offset.y,
        );

        let current_camera_pos = camera.position();
        let new_pos = current_camera_pos.lerp(&target_camera_pos, 0.05);

        camera.set_position(new_pos);
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        let player_pos = *ctx.render.get_mesh(self.player).position();
        let cam_pos = *ctx.render.camera().position();
        let fps = ctx.time.fps();

        ctx.render.set_active_layer(Layer::Ui);

        ctx.render
            .debug_text(format!("FPS: {:.0}", fps), 10.0, 10.0);

        ctx.render.debug_text(
            format!("Player: ({:.1}, {:.1})", player_pos.x, player_pos.y),
            10.0,
            30.0,
        );

        ctx.render.debug_text(
            format!("Camera: ({:.1}, {:.1})", cam_pos.x, cam_pos.y),
            10.0,
            50.0,
        );

        ctx.render
            .debug_text("Press space to start a camera shake!", 10.0, 70.0);

        ctx.render.debug_text(
            format!(
                "Allocated: {:.2} MB",
                ctx.profiling.mem.current() as f32 / 1024.0 / 1024.0
            ),
            10.0,
            90.0,
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
                    player: MeshHandle::dummy(),
                    corner_rects: Vec::new(),
                }),
        )
        .build()
        .run();
}

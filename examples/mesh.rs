use karna::{
    AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder,
    input::KeyCode,
    math::{Vector2, Vector3},
    render::{Color, Geometry, Layer, Material, Mesh, TextureKind, Transform3d},
    utils::Handle,
};

struct Demo {
    mesh: Handle<Mesh>,
    yaw: f32,
    pitch: f32,
}

impl Scene for Demo {
    fn load(&mut self, ctx: &mut Context) {
        ctx.time.set_target_fps(120);

        let mesh = Mesh::new(
            Geometry::cube(1.0),
            Material::new_color(Color::Cyan),
            Transform3d::default().with_position([0.0, 0.0, 0.0]),
        );
        self.mesh = ctx.scene.add_mesh(mesh);

        let mesh = Mesh::new(
            Geometry::circle(25.0, 32),
            Material::new_color(Color::Magenta),
            Transform3d::default()
                .with_position([600.0, 200.0, 0.0])
                .with_scale([5.0, 5.0, 0.0]),
        );
        ctx.scene.add_mesh(mesh);

        let cat = ctx
            .assets
            .load_image_bytes(include_bytes!("assets/cat.jpg").to_vec());
        let size = ctx.assets.get_image(cat).unwrap().size.to_f32();

        let mesh = Mesh::new(
            Geometry::unit_rect(),
            Material::new_texture(TextureKind::Full(cat)),
            Transform3d::default()
                .with_position([125.0, 200.0, 0.0])
                .with_scale(Vector2::from(size).extend(0.0)),
        );

        ctx.scene.add_mesh(mesh);
        ctx.window.capture_mouse(true);

        let camera = ctx.scene.camera_mut();

        camera.set_position([0.0, 0.0, 5.0]);
    }

    fn update(&mut self, ctx: &mut Context) {
        let vel = 10.0 * ctx.time.delta();

        let mouse_sensitivity = 0.003;
        let mouse_delta = ctx.input.mouse_delta();

        self.yaw += mouse_delta.x * mouse_sensitivity;
        self.pitch -= mouse_delta.y * mouse_sensitivity;

        self.pitch = self.pitch.clamp(-1.5, 1.5);

        let forward = Vector3::new(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vector3::new(self.yaw.cos(), 0.0, self.yaw.sin());

        let camera = ctx.scene.camera_mut();
        let pos = camera.position_mut();

        if ctx.input.key_held(&KeyCode::KeyW) {
            *pos += forward * vel;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            *pos -= forward * vel;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            *pos -= right * vel;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            *pos += right * vel;
        }

        if ctx.input.key_held(&KeyCode::Space) {
            pos.y += vel;
        }

        if ctx.input.key_held(&KeyCode::ShiftLeft) {
            pos.y -= vel;
        }

        if ctx.input.key_pressed(&KeyCode::KeyF) {
            if ctx.window.is_fullscreen() {
                ctx.window.set_windowed();
            } else {
                ctx.window.set_fullscreen();
            }
        }

        let look_target = *pos
            + Vector3::new(
                self.yaw.sin() * self.pitch.cos(),
                self.pitch.sin(),
                -self.yaw.cos() * self.pitch.cos(),
            );

        camera.look_at(look_target);

        let mesh = ctx.scene.get_mesh_mut(self.mesh).unwrap();

        *mesh.rotation_mut() += 0.01;
    }

    fn render(&mut self, ctx: &RenderContext, draw: &mut Draw) {
        draw.set_layer(Layer::Ui);
        draw.set_color(Color::White);

        draw.debug_text(format!("fps: {}", ctx.time.fps()), 10.0, 10.0);
        draw.debug_text(format!("dt: {}", ctx.time.delta()), 10.0, 30.0);
        draw.debug_text(
            format!(
                "Instance Writes: {}",
                ctx.profiling.render.instance_writes()
            ),
            10.0,
            50.0,
        );

        draw.debug_text(
            format!("Yaw: {:.2}, Pitch: {:.2}", self.yaw, self.pitch),
            10.0,
            70.0,
        );

        draw.set_layer(Layer::World);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("demo window")
                .with_label("main")
                .with_resizable(false)
                .with_size((800, 600))
                .with_initial_scene(Demo {
                    mesh: Handle::dummy(),
                    yaw: 0.0,
                    pitch: 0.0,
                }),
        )
        .build()
        .run();
}

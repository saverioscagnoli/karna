use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Handle;
use karna::assets::Image;
use karna::imgui::Condition;
use karna::math::Vector3;
use karna::math::Vector4;
use karna::render::Camera;
use karna::render::Color;
use karna::render::Draw;
use karna::render::Layer;
use karna::render::Material;
use karna::render::Mesh;
use karna::render::SceneRef;
use karna::render::cube;
use logging::Config;
use logging::LevelFilter;

struct Cubes {
    cyan: Handle<Mesh>,
    textured: Handle<Mesh>,
    cobblestone: Handle<Image>,
    colored_mat: Handle<Material>,
    camera: Handle<Camera>,
    fov: f32,
    right_color: Vector4<f32>,
}

impl Scene for Cubes {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);
        ctx.window.set_resizable(true);
        scene.set_clear_color(Color::rgb(0.07, 0.07, 0.09));

        let mut camera = Camera::perspective(ctx.window.pixel_size(), 75.0, 0.1, 100.0);
        camera.set_position([0.0, 0.0, -4.0]);
        camera.set_target(Vector3::zero());

        let camera = scene.add_camera(camera);
        scene.set_camera(Layer::World, camera);

        let (vertices, indices) = cube();
        let geometry = scene.create_geometry("cube", vertices, indices);

        let cyan_mat = scene.create_color_material(ctx.assets, Color::Cyan);

        let cobblestone = ctx.assets.load_image("assets/cobblestone.png");
        let cobblestone_mat = scene.create_textured_material(ctx.assets, cobblestone, Color::White);

        let mut cyan = Mesh::new(geometry, cyan_mat);
        cyan.move_x(-2.0);

        let cyan = scene.add_mesh(cyan);

        let mut textured = Mesh::new(geometry, cobblestone_mat);
        textured.move_x(2.0);

        let textured = scene.add_mesh(textured);

        Self {
            cyan,
            textured,
            cobblestone,
            colored_mat: cyan_mat,
            camera,
            fov: 75.0,
            right_color: Color::White.into(),
        }
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        let dt = ctx.time.delta();

        scene.edit_material(ctx.assets, self.colored_mat, |desc| {
            desc.base_color = self.right_color.into();
        });

        let colored = scene.get_mesh_mut(self.cyan);

        colored.rotation_mut().x += dt;
        colored.rotation_mut().y -= dt;
        colored.rotation_mut().z += dt;

        let textured = scene.get_mesh_mut(self.textured);

        textured.rotation_mut().x -= dt;
        textured.rotation_mut().y += dt;
        textured.rotation_mut().z -= dt;

        let camera = scene.get_camera_mut(self.camera);

        camera.set_fov_degrees(self.fov);
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.set_layer(Layer::Ui);
        draw.debug_text(format!("fps {}", ctx.time.fps()), 12.0, 12.0);

        draw.imgui(|ui| {
            ui.window("Cubes Example")
                .size((300.0, 200.0), Condition::FirstUseEver)
                .build(|ui| {
                    ui.slider_f32("Camera FOV", &mut self.fov, 1.0, 120.0);
                    ui.color_edit4("Right Cube Color", &mut self.right_color);
                })
        });
    }
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_scene::<Cubes>(0)
                .with_active_scene(0),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}

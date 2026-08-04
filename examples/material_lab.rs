// Live material editing: one cobblestone-textured cube whose MaterialDesc is
// mutated every frame from an imgui panel via `SceneRef::edit_material`.
//
// base_color, metallic, alpha_mode and double_sided actually change what's
// drawn — the mesh shader shades a metallic-roughness-ish tint/specular
// split (fixed glossiness, no roughness input yet) over the sampled
// texture. The rest of the PBR fields (roughness, emissive, the extra
// maps, ...) are real, stored on the material, and round-trip through
// `edit_material` correctly, but nothing shades with them yet, so their
// sliders are inert on screen. The panel says so.

use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Handle;
use karna::imgui::Condition;
use karna::math::Vector3;
use karna::math::Vector4;
use karna::render::AlphaMode;
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

struct MaterialLab {
    cube: Handle<Mesh>,
    material: Handle<Material>,
    camera: Handle<Camera>,

    base_color: Vector4<f32>,
    alpha_blend: bool,
    double_sided: bool,

    metallic: f32,
    roughness: f32,
    reflectance: f32,
    emissive: Vector4<f32>,
    emissive_strength: f32,
    unlit: bool,
}

impl Scene for MaterialLab {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);
        scene.set_clear_color(Color::rgb(0.07, 0.07, 0.09));

        let mut camera = Camera::perspective(ctx.window.pixel_size(), 60.0, 0.1, 100.0);
        camera.set_position([0.0, 0.0, -3.5]);
        camera.set_target(Vector3::zero());

        let camera = scene.add_camera(camera);
        scene.set_camera(Layer::World, camera);

        let (vertices, indices) = cube();
        let geometry = scene.create_geometry("cube", vertices, indices);

        let material = scene.create_color_material(ctx.assets, Color::White);

        let cube = scene.add_mesh(Mesh::new(geometry, material));

        Self {
            cube,
            material,
            camera,

            base_color: Color::White.into(),
            alpha_blend: false,
            double_sided: false,

            metallic: 0.0,
            roughness: 1.0,
            reflectance: 0.5,
            emissive: Color::Black.into(),
            emissive_strength: 0.0,
            unlit: false,
        }
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        let dt = ctx.time.delta();

        let cube = scene.get_mesh_mut(self.cube);

        cube.rotate_y(dt * 0.5);
        cube.rotate_x(dt * 0.2);

        scene.edit_material(ctx.assets, self.material, |desc| {
            desc.base_color = self.base_color.into();
            desc.alpha_mode = if self.alpha_blend {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
            desc.double_sided = self.double_sided;

            desc.metallic = self.metallic;
            desc.roughness = self.roughness;
            desc.reflectance = self.reflectance;
            desc.emissive = self.emissive.into();
            desc.emissive_strength = self.emissive_strength;
            desc.unlit = self.unlit;
        });

        let _ = scene.get_camera_mut(self.camera);
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.set_layer(Layer::Ui);
        draw.debug_text(format!("fps {}", ctx.time.fps()), 12.0, 12.0);

        draw.imgui(|ui| {
            ui.window("Material Lab")
                .size((340.0, 420.0), Condition::FirstUseEver)
                .build(|ui| {
                    ui.separator_text("Live (actually shaded)");
                    ui.color_edit4("Base color", &mut self.base_color);
                    ui.slider_f32("Metallic", &mut self.metallic, 0.0, 1.0);
                    ui.checkbox("Alpha blend", &mut self.alpha_blend);
                    ui.checkbox("Double sided", &mut self.double_sided);

                    if self.alpha_blend {
                        ui.text("Lower base color alpha and toggle double");
                        ui.text("sided to see the back faces through it.");
                    }

                    ui.separator_text("Stored (no shader reads these yet)");
                    ui.slider_f32("Roughness", &mut self.roughness, 0.0, 1.0);
                    ui.slider_f32("Reflectance", &mut self.reflectance, 0.0, 1.0);
                    ui.color_edit4("Emissive", &mut self.emissive);
                    ui.slider_f32("Emissive strength", &mut self.emissive_strength, 0.0, 10.0);
                    ui.checkbox("Unlit", &mut self.unlit);
                });
        });
    }
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Material Lab")
                .with_size((1280, 720))
                .with_scene::<MaterialLab>(0)
                .with_active_scene(0),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}

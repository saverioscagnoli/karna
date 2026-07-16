use karna::prelude::*;

struct Res {
    cube_mat: Handle<Material>,
    cube_color: [f32; 4],
    cube_pos: [f32; 3],
    cube: Handle<Mesh>,
}

struct CubeDemo;

impl Scene for CubeDemo {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let view = ctx.window.size();
        let fov = 75.0;
        let near = 1.0;
        let far = 1000.0;

        scene.set_camera_projection(Projection::standard_3d(view, fov, near, far));

        let geometry = scene.add_geometry(Geometry::cube_sized(3.0));
        let material = scene.add_material(MaterialDesc::default().color(Color::Cyan));

        let mut mesh = Mesh::new(geometry, material);

        // The default camera sits at (0, 0, -5) looking toward +z,
        let pos = Vector3::new(0.0, 0.0, 5.0);

        mesh.position = pos.into();

        let cube = scene.add_mesh(mesh);

        let res = Res {
            cube,
            cube_mat: material,
            cube_color: Color::Cyan.into(),
            cube_pos: pos.into(),
        };

        ctx.time.set_target_fps(120);
        ctx.resources.insert(res);
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let dt = ctx.time.delta();
        let res = ctx.resources.get::<Res>();

        let mat = scene.get_material_mut(res.cube_mat);

        mat.set_base_color(res.cube_color.into());

        let cube = scene.get_mesh_mut(res.cube);

        cube.position = res.cube_pos.into();
        cube.rotation.x += 0.8 * dt;
        cube.rotation.y += 1.2 * dt;
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        let res = ctx.resources.get_mut::<Res>();

        draw.imgui(|ui| {
            ui.window("Cube panel")
                .size([300.0, 400.0], imgui::Condition::FirstUseEver)
                .position([10.0, 10.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.color_picker4_config("cube", &mut res.cube_color)
                        .picker_mode(imgui::ColorPickerMode::HueWheel)
                        .build();

                    ui.slider_float3("cube x", &mut res.cube_pos, -5.0, 5.0);
                });
        });
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Cube demo")
                .with_size((1280, 720))
                .with_scene("cube", CubeDemo)
                .with_active_scene("cube"),
        )
        .build()
        .run()
}

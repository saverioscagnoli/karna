//! Port of sokol's imgui-images-sapp: a spinning cube is drawn into a small
//! offscreen canvas, which imgui then displays with different samplers.
//!
//! The engine has no depth buffer or 3D pipeline, so the cube is projected on
//! the CPU and only its (convex, so always correct) front faces are drawn.

use karna::App;
use karna::Context;
use karna::Scene;
use karna::SceneView;
use karna::WindowBuilder;
use karna::imgui::Condition;
use karna::imgui::Image;
use karna::logging::Config;
use karna::math::Matrix4;
use karna::math::Vector2;
use karna::math::Vector3;
use karna::math::Vector4;
use karna::render::Canvas;
use karna::render::Draw;
use karna::render::SamplerKind;

const CANVAS_SIZE: u32 = 32;

/// Cube faces as (outward normal, corners, color), colors as in the sokol
/// example.
const FACES: [([f32; 3], [[f32; 3]; 4], [f32; 3]); 6] = [
    (
        [0.0, 0.0, -1.0],
        [[-1.0, 1.0, -1.0], [1.0, 1.0, -1.0], [1.0, -1.0, -1.0], [-1.0, -1.0, -1.0]],
        [1.0, 0.0, 0.0],
    ),
    (
        [0.0, 0.0, 1.0],
        [[-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0]],
        [0.0, 1.0, 0.0],
    ),
    (
        [-1.0, 0.0, 0.0],
        [[-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0], [-1.0, 1.0, -1.0], [-1.0, -1.0, -1.0]],
        [0.0, 0.0, 1.0],
    ),
    (
        [1.0, 0.0, 0.0],
        [[1.0, -1.0, 1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [1.0, 1.0, 1.0]],
        [1.0, 0.5, 0.0],
    ),
    (
        [0.0, -1.0, 0.0],
        [[1.0, -1.0, -1.0], [1.0, -1.0, 1.0], [-1.0, -1.0, 1.0], [-1.0, -1.0, -1.0]],
        [0.0, 0.5, 1.0],
    ),
    (
        [0.0, 1.0, 0.0],
        [[-1.0, 1.0, -1.0], [-1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, -1.0]],
        [1.0, 0.0, 0.5],
    ),
];

struct ImguiImages {
    canvas: Canvas,
    angle: f32,
}

impl Scene for ImguiImages {
    fn load(&mut self, ctx: &mut Context, _scene: &mut SceneView) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: &mut Context, _scene: &mut SceneView) {
        self.angle += ctx.time.delta() * 60.0;
    }

    fn draw(&mut self, _ctx: &mut Context, draw: &mut Draw) {
        draw.set_clear_color([0.5, 0.5, 1.0, 1.0]);

        let a = self.angle.to_radians();
        let eye = Vector3::new(a.sin() * 4.0, a.sin() * 2.0, a.cos() * 4.0);
        let view = Matrix4::look_at(&eye, &Vector3::zero(), &Vector3::new(0.0, 1.0, 0.0));
        let proj = Matrix4::perspective(45f32.to_radians(), 1.0, 0.1, 100.0);
        let view_proj = proj.matmul(&view);

        let project = |p: [f32; 3]| {
            let clip = view_proj.mul_vec(&Vector4::new(p[0], p[1], p[2], 1.0));
            let side = CANVAS_SIZE as f32;

            Vector2::new(
                (clip.x / clip.w * 0.5 + 0.5) * side,
                (0.5 - clip.y / clip.w * 0.5) * side,
            )
        };

        draw.canvas(&self.canvas, |d| {
            for (normal, corners, color) in FACES {
                let normal = Vector3::from(normal);

                // The cube is convex and centered at the origin (each face
                // center equals its normal), so facing the eye means visible.
                if normal.dot(&(eye - normal)) <= 0.0 {
                    continue;
                }

                d.set_color([color[0], color[1], color[2], 1.0]);
                d.quad(corners.map(project));
            }
        });

        let canvas = &self.canvas;

        draw.imgui(|ui| {
            ui.window("karna + Dear ImGui Image Test")
                .size([540.0, 560.0], Condition::Once)
                .position([20.0, 20.0], Condition::Once)
                .build(|| {
                    let size = [256.0, 256.0];

                    Image::new(ui, canvas.texture_id(SamplerKind::NearestClamp), size).build();
                    ui.same_line_with_spacing(0.0, 4.0);
                    Image::new(ui, canvas.texture_id(SamplerKind::LinearClamp), size).build();

                    Image::new(ui, canvas.texture_id(SamplerKind::NearestRepeat), size)
                        .uv1([4.0, 4.0])
                        .build();
                    ui.same_line_with_spacing(0.0, 4.0);
                    Image::new(ui, canvas.texture_id(SamplerKind::LinearMirror), size)
                        .uv1([4.0, 4.0])
                        .build();
                });
        });
    }
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("imgui images")
                .with_size((580, 600))
                .with_resizable(true)
                .with_scene(
                    0,
                    ImguiImages {
                        canvas: Canvas::new((CANVAS_SIZE, CANVAS_SIZE)),
                        angle: 0.0,
                    },
                )
                .with_active_scene(0),
        )
        .build()
        .run();
}

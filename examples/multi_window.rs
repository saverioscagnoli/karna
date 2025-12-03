use karna::{
    App, Context, Scene,
    render::{Color, Mesh, MeshGeometry, Transform2D},
};
use renderer::material::Material;

struct SimpleScene {
    mesh: Mesh,
}

impl SimpleScene {
    fn new(color: Color) -> Self {
        Self {
            mesh: Mesh::new(
                MeshGeometry::circle(50.0, 32),
                Material::new_color(color),
                Transform2D::default().with_position([400.0, 300.0]),
            ),
        }
    }
}

impl Scene for SimpleScene {
    fn load(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        let t = ctx.time.elapsed().as_secs_f32();
        self.mesh.position.x = 400.0 + t.sin() * 100.0;
        self.mesh.position.y = 300.0 + t.cos() * 100.0;
    }

    fn render(&mut self, ctx: &mut Context) {
        self.mesh.render(&mut ctx.render);
    }
}

fn main() {
    App::new()
        // Register a scene that we will use for the second window
        .with_scene("blue_scene", Box::new(SimpleScene::new(Color::Blue)))
        // Create the first window with its own scene
        .with_initial_scene("red_scene", Box::new(SimpleScene::new(Color::Red)))
        // Create the second window using the registered "blue_scene"
        .with_window("blue_scene")
        .run();
}

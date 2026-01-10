use karna::{
    AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder,
    assets::{Font, Image},
    input::KeyCode,
    math::Vector2,
    render::Color,
    utils::Handle,
};
use renderer::{Geometry, Material, Mesh, Projection, TextureKind, Transform3d};

#[derive(Default)]
struct Demo {
    cat: Handle<Mesh>,
}

impl Scene for Demo {
    fn load(&mut self, ctx: &mut Context) {
        let handle = ctx
            .assets
            .load_image_bytes(include_bytes!("assets/cat.png").to_vec());

        let meta = ctx.assets.get_image(handle);

        let mesh = Mesh::new(
            Geometry::rect(meta.size.to_f32()),
            Material::new_texture(TextureKind::Full(handle)),
            Transform3d::default().with_position([400.0, 300.0, 0.0]),
        );

        ctx.scene.add_mesh(mesh);
    }

    fn update(&mut self, ctx: &mut Context) {}

    fn render(&mut self, ctx: &RenderContext, draw: &mut Draw) {}
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("demo window")
                .with_label("main")
                .with_resizable(false)
                .with_size((800, 600))
                .with_initial_scene(Demo::default()),
        )
        .build()
        .run();
}

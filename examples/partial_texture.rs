use karna::{
    AppBuilder, Context, Scene, WindowBuilder, label,
    render::{Color, Geometry, Material, Mesh, TextureKind, Transform},
};

pub struct TextureRegionDemo {
    full_texture: Mesh,
    partial_top_left: Mesh,
    partial_top_right: Mesh,
    partial_bottom_left: Mesh,
    partial_bottom_right: Mesh,
}

impl Scene for TextureRegionDemo {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::rgb(0.1, 0.1, 0.15));

        ctx.assets.load_image(
            label!("sprite_sheet"),
            include_bytes!("assets/cat.jpg").to_vec(),
        );
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {
        // Render the full texture
        ctx.render.draw_mesh(&self.full_texture);

        // Render the four quadrants as separate meshes
        ctx.render.draw_mesh(&self.partial_top_left);
        ctx.render.draw_mesh(&self.partial_top_right);
        ctx.render.draw_mesh(&self.partial_bottom_left);
        ctx.render.draw_mesh(&self.partial_bottom_right);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("partial texture demo")
                .with_size((1280, 720))
                .with_resizable(false)
                .with_initial_scene(TextureRegionDemo {
                    // Full texture on the left side
                    full_texture: Mesh::new(
                        Geometry::rect(256.0, 256.0),
                        Material::new_texture(TextureKind::Full(label!("sprite_sheet"))),
                        Transform::default().with_position([50.0, 50.0]),
                    ),
                    // Top-left quadrant (0, 0, 128x128)
                    partial_top_left: Mesh::new(
                        Geometry::rect(128.0, 128.0),
                        Material::new_texture(TextureKind::Partial(
                            label!("sprite_sheet"),
                            0.0,
                            0.0,
                            128.0,
                            128.0,
                        )),
                        Transform::default().with_position([400.0, 50.0]),
                    ),
                    // Top-right quadrant (128, 0, 128x128)
                    partial_top_right: Mesh::new(
                        Geometry::rect(128.0, 128.0),
                        Material::new_texture(TextureKind::Partial(
                            label!("sprite_sheet"),
                            128.0,
                            0.0,
                            128.0,
                            128.0,
                        )),
                        Transform::default().with_position([550.0, 50.0]),
                    ),
                    // Bottom-left quadrant (0, 128, 128x128)
                    partial_bottom_left: Mesh::new(
                        Geometry::rect(128.0, 128.0),
                        Material::new_texture(TextureKind::Partial(
                            label!("sprite_sheet"),
                            0.0,
                            128.0,
                            128.0,
                            128.0,
                        )),
                        Transform::default().with_position([400.0, 200.0]),
                    ),
                    // Bottom-right quadrant (128, 128, 128x128)
                    partial_bottom_right: Mesh::new(
                        Geometry::rect(128.0, 128.0),
                        Material::new_texture(TextureKind::Partial(
                            label!("sprite_sheet"),
                            128.0,
                            128.0,
                            128.0,
                            128.0,
                        )),
                        Transform::default().with_position([550.0, 200.0]),
                    ),
                }),
        )
        .build()
        .run();
}

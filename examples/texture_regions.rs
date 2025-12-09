use karna::{
    AppBuilder, Context, Scene, WindowBuilder, label,
    render::{Color, Material, Mesh, MeshGeometry, TextureKind, TextureRegion, Transform},
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

        // Load a texture (assuming it's at least 256x256 pixels)
        ctx.render
            .load_texture(label!("sprite_sheet"), include_bytes!("assets/cat.jpg"));
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
                .with_title("Texture Regions Demo - Full vs Partial Textures")
                .with_size((1280, 720))
                .with_initial_scene(Box::new(TextureRegionDemo {
                    // Full texture on the left side
                    full_texture: Mesh {
                        geometry: MeshGeometry::rect(),
                        material: Material {
                            texture: Some(TextureKind::Full(label!("sprite_sheet"))),
                            color: None,
                        },
                        transform: Transform::default()
                            .with_position([50.0, 50.0])
                            .with_scale([256.0, 256.0]),
                    },
                    // Top-left quadrant (0, 0, 128x128)
                    partial_top_left: Mesh {
                        geometry: MeshGeometry::rect(),
                        material: Material {
                            texture: Some(TextureKind::Partial(
                                label!("sprite_sheet"),
                                TextureRegion::new(0, 0, 128, 128),
                            )),
                            color: None,
                        },
                        transform: Transform::default()
                            .with_position([400.0, 50.0])
                            .with_scale([128.0, 128.0]),
                    },
                    // Top-right quadrant (128, 0, 128x128)
                    partial_top_right: Mesh {
                        geometry: MeshGeometry::rect(),
                        material: Material {
                            texture: Some(TextureKind::Partial(
                                label!("sprite_sheet"),
                                TextureRegion::new(128, 0, 128, 128),
                            )),
                            color: None,
                        },
                        transform: Transform::default()
                            .with_position([550.0, 50.0])
                            .with_scale([128.0, 128.0]),
                    },
                    // Bottom-left quadrant (0, 128, 128x128)
                    partial_bottom_left: Mesh {
                        geometry: MeshGeometry::rect(),
                        material: Material {
                            texture: Some(TextureKind::Partial(
                                label!("sprite_sheet"),
                                TextureRegion::new(0, 128, 128, 128),
                            )),
                            color: None,
                        },
                        transform: Transform::default()
                            .with_position([400.0, 200.0])
                            .with_scale([128.0, 128.0]),
                    },
                    // Bottom-right quadrant (128, 128, 128x128)
                    partial_bottom_right: Mesh {
                        geometry: MeshGeometry::rect(),
                        material: Material {
                            texture: Some(TextureKind::Partial(
                                label!("sprite_sheet"),
                                TextureRegion::new(128, 128, 128, 128),
                            )),
                            color: None,
                        },
                        transform: Transform::default()
                            .with_position([550.0, 200.0])
                            .with_scale([128.0, 128.0]),
                    },
                })),
        )
        .build()
        .run();
}

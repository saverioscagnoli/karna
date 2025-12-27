mod camera;
mod color;
mod immediate;
mod layer;
mod mesh;
mod shader;
mod sprite;
mod text;

use crate::{mesh::MeshInstanceGpu, shader::Shader};
use assets::AssetManager;
use macros::{Get, Set};
use math::{Size, Vector2};
use std::sync::Arc;
use traccia::info;
use utils::Handle;
use winit::window::Window;

// Re-exports
pub use camera::{Camera, Projection};
pub use color::Color;
pub use layer::{Layer, RenderLayer};
pub use mesh::{
    Mesh, MeshHandle, Vertex,
    geometry::Geometry,
    material::{Material, TextureKind},
    transform::Transform,
};
pub use sprite::{Frame, Sprite, SpriteHandle};
pub use text::{Text, TextHandle};

pub(crate) trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[derive(Get, Set)]
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    assets: Arc<AssetManager>,

    #[get]
    #[set(into)]
    clear_color: Color,

    #[get]
    #[set(into)]
    draw_color: Color,

    // Render layers
    world: RenderLayer,
    ui: RenderLayer,
    user_layers: Vec<RenderLayer>,

    #[get(copied)]
    #[set]
    active_layer: Layer,

    retained_pipeline: wgpu::RenderPipeline,
    immediate_pipeline: wgpu::RenderPipeline,

    // Wireframe pipeline
    wireframe_pipeline: wgpu::RenderPipeline,
    wireframe_toggle: bool,
}

impl Renderer {
    #[doc(hidden)]
    pub fn new(window: Arc<Window>, assets: Arc<AssetManager>) -> Self {
        let gpu = gpu::get();
        let size = window.inner_size();

        let surface = gpu
            .instance()
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let surface_caps = surface.get_capabilities(gpu.adapter());
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(gpu.device(), &config);

        let camera = Camera::new(Projection::Orthographic {
            left: 0.0,
            right: size.width as f32,
            bottom: size.height as f32,
            top: 0.0,
            z_near: -1.0,
            z_far: 1.0,
        });

        let ui_camera = Camera::new(Projection::Orthographic {
            left: 0.0,
            right: size.width as f32,
            bottom: size.height as f32,
            top: 0.0,
            z_near: -1.0,
            z_far: 1.0,
        });

        let shader =
            Shader::from_wgsl_file(include_str!("../../shaders/basic_2d.wgsl"), Some("shader"));

        let immediate_shader = Shader::from_wgsl_file(
            include_str!("../../shaders/immediate.wgsl"),
            Some("immediate_shader"),
        );

        let retained_pipeline = shader
            .pipeline_builder()
            .label("triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc(), MeshInstanceGpu::desc()],
            );

        let immediate_pipeline = immediate_shader
            .pipeline_builder()
            .label("immediate pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc()], // Only Vertex, no MeshInstanceGpu!
            );

        let wireframe_pipeline = shader
            .pipeline_builder()
            .label("wireframe triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleStrip)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .polygon_mode(wgpu::PolygonMode::Line)
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc(), MeshInstanceGpu::desc()],
            );

        let world = RenderLayer::new(surface_format, camera, assets.clone());
        let ui = RenderLayer::new(surface_format, ui_camera, assets.clone());

        Self {
            surface,
            config,
            assets: assets.clone(),
            clear_color: Color::Black,
            draw_color: Color::White,
            world,
            ui,
            active_layer: Layer::World,
            user_layers: Vec::new(),
            retained_pipeline,
            immediate_pipeline,
            wireframe_pipeline,
            wireframe_toggle: false,
        }
    }

    #[inline]
    /// Gets adapter information
    pub fn info() -> wgpu::AdapterInfo {
        gpu::adapter().get_info()
    }

    #[inline]
    #[doc(hidden)]
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        info!("Resized to {}x{}", width, height);

        self.surface.configure(&gpu::device(), &self.config);
        self.world.resize(width, height);

        self.config.width = width;
        self.config.height = height;
    }

    #[inline]
    pub fn add_mesh(&mut self, layer: Layer, mesh: Mesh) -> Handle<Mesh> {
        match layer {
            Layer::World => self.world.add_mesh(mesh),
            Layer::Ui => self.ui.add_mesh(mesh),
            Layer::N(i) => self.user_layers[i].add_mesh(mesh),
        }
    }

    #[inline]
    pub fn get_mesh(&self, id: Handle<Mesh>) -> &Mesh {
        match self.active_layer {
            Layer::World => self.world.get_mesh(id),
            Layer::Ui => self.ui.get_mesh(id),
            Layer::N(i) => self.user_layers[i].get_mesh(id),
        }
    }

    #[inline]
    pub fn get_mesh_mut(&mut self, id: Handle<Mesh>) -> &mut Mesh {
        match self.active_layer {
            Layer::World => self.world.get_mesh_mut(id),
            Layer::Ui => self.ui.get_mesh_mut(id),
            Layer::N(i) => self.user_layers[i].get_mesh_mut(id),
        }
    }

    #[inline]
    pub fn remove_mesh(&mut self, layer: Layer, id: Handle<Mesh>) {
        match layer {
            Layer::World => self.world.remove_mesh(id),
            Layer::Ui => self.ui.remove_mesh(id),
            Layer::N(i) => self.user_layers[i].remove_mesh(id),
        }
    }

    #[inline]
    pub fn add_text(&mut self, layer: Layer, text: Text) -> Handle<Text> {
        match layer {
            Layer::World => self.world.add_text(text),
            Layer::Ui => self.ui.add_text(text),
            Layer::N(i) => self.user_layers[i].add_text(text),
        }
    }

    #[inline]
    pub fn get_text(&self, id: Handle<Text>) -> &Text {
        match self.active_layer {
            Layer::World => self.world.get_text(id),
            Layer::Ui => self.ui.get_text(id),
            Layer::N(i) => self.user_layers[i].get_text(id),
        }
    }

    #[inline]
    pub fn get_text_mut(&mut self, id: Handle<Text>) -> &mut Text {
        match self.active_layer {
            Layer::World => self.world.get_text_mut(id),
            Layer::Ui => self.ui.get_text_mut(id),
            Layer::N(i) => self.user_layers[i].get_text_mut(id),
        }
    }

    #[inline]
    pub fn remove_text(&mut self, layer: Layer, id: Handle<Text>) {
        match layer {
            Layer::World => self.world.remove_text(id),
            Layer::Ui => self.ui.remove_text(id),
            Layer::N(i) => self.user_layers[i].remove_text(id),
        }
    }

    #[inline]
    pub fn add_sprite(&mut self, layer: Layer, sprite: Sprite) -> Handle<Sprite> {
        match layer {
            Layer::World => self.world.add_sprite(sprite),
            Layer::Ui => self.ui.add_sprite(sprite),
            Layer::N(i) => self.user_layers[i].add_sprite(sprite),
        }
    }

    #[inline]
    pub fn get_sprite(&self, id: Handle<Sprite>) -> &Sprite {
        match self.active_layer {
            Layer::World => self.world.get_sprite(id),
            Layer::Ui => self.ui.get_sprite(id),
            Layer::N(i) => self.user_layers[i].get_sprite(id),
        }
    }

    #[inline]
    pub fn get_sprite_mut(&mut self, id: Handle<Sprite>) -> &mut Sprite {
        match self.active_layer {
            Layer::World => self.world.get_sprite_mut(id),
            Layer::Ui => self.ui.get_sprite_mut(id),
            Layer::N(i) => self.user_layers[i].get_sprite_mut(id),
        }
    }

    #[inline]
    pub fn remove_sprite(&mut self, id: Handle<Sprite>) {
        match self.active_layer {
            Layer::World => self.world.remove_sprite(id),
            Layer::Ui => self.ui.remove_sprite(id),
            Layer::N(i) => self.user_layers[i].remove_sprite(id),
        }
    }

    // === Immediate Rendering ===
    pub fn fill_rect<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<Vector2>,
        S: Into<Size<f32>>,
    {
        let pos = pos.into();
        let size = size.into();
        let draw_color = self.draw_color.into();

        match self.active_layer {
            Layer::World => self.world.immediate.fill_rect(pos, size, draw_color),
            Layer::Ui => self.ui.immediate.fill_rect(pos, size, draw_color),
            Layer::N(i) => self.user_layers[i]
                .immediate
                .fill_rect(pos, size, draw_color),
        }
    }

    #[inline]
    pub fn toggle_wireframe(&mut self) {
        self.wireframe_toggle = !self.wireframe_toggle;
    }

    #[inline]
    #[doc(hidden)]
    pub fn present(&mut self, dt: f32) -> Result<(), wgpu::SurfaceError> {
        let gpu = gpu::get();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.world.update(self.config.width, self.config.height, dt);

        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            let pipeline = if self.wireframe_toggle {
                &self.wireframe_pipeline
            } else {
                &self.retained_pipeline
            };

            self.world
                .present(&mut render_pass, pipeline, &self.immediate_pipeline);
        }

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

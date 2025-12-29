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
use globals::profiling;
use macros::{Get, Set};
use math::{Size, Vector2};
use std::sync::Arc;
use traccia::info;
use utils::{Handle, Label, label};
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

    #[get(name = "font")]
    #[set(name = "set_font")]
    current_font: Label,

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
            current_font: label!("debug"),
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

    /// Gets adapter information
    #[inline]
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
        self.ui.resize(width, height);

        for layer in self.user_layers.iter_mut() {
            layer.resize(width, height);
        }

        self.config.width = width;
        self.config.height = height;
    }

    #[inline]
    fn render_layer(&self, layer: Layer) -> &RenderLayer {
        match layer {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::N(i) => &self.user_layers[i],
        }
    }

    #[inline]
    fn render_layer_mut(&mut self, layer: Layer) -> &mut RenderLayer {
        match layer {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::N(i) => &mut self.user_layers[i],
        }
    }

    #[inline]
    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshHandle {
        let layer = self.active_layer;
        let handle = self.render_layer_mut(layer).add_mesh(mesh);

        MeshHandle { handle, layer }
    }

    #[inline]
    pub fn get_mesh(&self, handle: MeshHandle) -> &Mesh {
        self.render_layer(handle.layer).get_mesh(*handle)
    }

    #[inline]
    pub fn get_mesh_mut(&mut self, handle: MeshHandle) -> &mut Mesh {
        self.render_layer_mut(handle.layer).get_mesh_mut(*handle)
    }

    #[inline]
    pub fn remove_mesh(&mut self, handle: MeshHandle) {
        self.render_layer_mut(handle.layer).remove_mesh(*handle);
    }

    #[inline]
    pub fn add_text(&mut self, text: Text) -> TextHandle {
        let layer = self.active_layer;
        let handle = self.render_layer_mut(layer).add_text(text);

        TextHandle { layer, handle }
    }

    #[inline]
    pub fn get_text(&self, handle: TextHandle) -> &Text {
        self.render_layer(handle.layer).get_text(*handle)
    }

    #[inline]
    pub fn get_text_mut(&mut self, handle: TextHandle) -> &mut Text {
        self.render_layer_mut(handle.layer).get_text_mut(*handle)
    }

    #[inline]
    pub fn remove_text(&mut self, handle: TextHandle) {
        self.render_layer_mut(handle.layer).remove_text(*handle);
    }

    #[inline]
    pub fn add_sprite(&mut self, sprite: Sprite) -> SpriteHandle {
        let layer = self.active_layer;
        let handle = self.render_layer_mut(layer).add_sprite(sprite);

        SpriteHandle { layer, handle }
    }

    #[inline]
    pub fn get_sprite(&self, handle: SpriteHandle) -> &Sprite {
        self.render_layer(handle.layer).get_sprite(*handle)
    }

    #[inline]
    pub fn get_sprite_mut(&mut self, handle: SpriteHandle) -> &mut Sprite {
        self.render_layer_mut(handle.layer).get_sprite_mut(*handle)
    }

    #[inline]
    pub fn remove_sprite(&mut self, handle: SpriteHandle) {
        self.render_layer_mut(handle.layer).remove_sprite(*handle);
    }

    #[inline]
    pub fn camera(&self) -> &Camera {
        &self.render_layer(self.active_layer).camera
    }

    #[inline]
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.render_layer_mut(self.active_layer).camera
    }

    // === Immediate Rendering ===
    //
    // Separates retained rendering functionality (Above, add once, render until removed)
    // from immediate rendering functionality (Below, add once, render once)
    //
    // Immediate rendering is useful for quickly drawing shapes or text
    // without the complexity of retained rendering.

    /// Draws a filled rectangle in immediate rendering mode.
    #[inline]
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let pos = Vector2::new(x, y);
        let size = Size::new(w, h);
        let color = self.draw_color.into();

        self.render_layer_mut(self.active_layer)
            .immediate
            .fill_rect(pos, size, color);
    }

    /// Draws a filled rectangle in immediate rendering mode.
    /// Uses [`math::Vector2`] and [`math::Size`] as arguments.
    #[inline]
    pub fn fill_rect_v<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<Vector2>,
        S: Into<Size<f32>>,
    {
        let pos = pos.into();
        let size = size.into();
        let color = self.draw_color.into();

        self.render_layer_mut(self.active_layer)
            .immediate
            .fill_rect(pos, size, color);
    }

    #[inline]
    pub fn draw_text<T: Into<String>>(&mut self, text: T, x: f32, y: f32) {
        let font_label = self.current_font;
        let text = text.into();
        let color = self.draw_color.into();

        self.render_layer_mut(self.active_layer)
            .immediate
            .draw_text(font_label, text, x, y, color);
    }

    #[inline]
    pub fn draw_text_v<T, P>(&mut self, text: T, pos: P)
    where
        T: Into<String>,
        P: Into<Vector2>,
    {
        let font_label = self.current_font;
        let text = text.into();
        let pos = pos.into();
        let color = self.draw_color.into();

        self.render_layer_mut(self.active_layer)
            .immediate
            .draw_text(font_label, text, pos.x, pos.y, color);
    }

    #[inline]
    pub fn debug_text<T: Into<String>>(&mut self, text: T, x: f32, y: f32) {
        let text = text.into();
        let color = self.draw_color.into();

        self.render_layer_mut(self.active_layer)
            .immediate
            .debug_text(text, x, y, color);
    }

    #[inline]
    pub fn debug_text_v<T, P>(&mut self, text: T, pos: P)
    where
        T: Into<String>,
        P: Into<Vector2>,
    {
        let pos = pos.into();
        let color = self.draw_color.into();

        self.render_layer_mut(self.active_layer)
            .immediate
            .debug_text(text.into(), pos.x, pos.y, color);
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
        self.ui.update(self.config.width, self.config.height, dt);

        for layer in self.user_layers.iter_mut() {
            layer.update(self.config.width, self.config.height, dt);
        }

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

            self.ui
                .present(&mut render_pass, pipeline, &self.immediate_pipeline);

            for layer in self.user_layers.iter_mut() {
                layer.present(&mut render_pass, pipeline, &self.immediate_pipeline);
            }
        }

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

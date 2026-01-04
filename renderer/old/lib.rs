mod camera;
mod color;
mod immediate;
mod layer;
mod mesh;
mod retained;
mod shader;
mod sprite;

use crate::{
    mesh::MeshInstanceGpu,
    retained::{GlyphInstance, TextVertex},
    shader::Shader,
};
use assets::AssetManager;
use logging::{LogError, LogLevel, info, warn};
use macros::{Get, Set};
use math::{Size, Vector2, Vector4};
use std::sync::{Arc, RwLock};
use utils::{Label, label};
use winit::window::Window;

// Re-exports
pub use camera::{Camera, Projection};
pub use color::Color;
pub use layer::{Layer, RenderLayer};
pub use mesh::{
    Mesh, MeshHandle, Vertex,
    geometry::Geometry,
    group::MeshGroup,
    material::{Material, TextureKind},
    transform::Transform,
};
pub use retained::{Text, TextHandle};
pub use sprite::{Frame, Sprite, SpriteHandle};

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
    text_pipeline: wgpu::RenderPipeline,

    depth_texture: wgpu::Texture,

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
            z_near: -1000.0,
            z_far: 1000.0,
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

        let text_shader =
            Shader::from_wgsl_file(include_str!("../../shaders/text.wgsl"), Some("text shader"));

        let retained_pipeline = shader
            .pipeline_builder()
            .label("triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .cull_mode(wgpu::Face::Back)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc(), MeshInstanceGpu::desc()],
            );

        let text_pipeline = text_shader
            .pipeline_builder()
            .label("text pipeline")
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
                &[TextVertex::desc(), GlyphInstance::desc()],
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

        let depth_texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float, // Standard depth format
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

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
            text_pipeline,
            wireframe_pipeline,
            wireframe_toggle: false,
            depth_texture,
        }
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

        self.depth_texture = gpu::device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
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
    pub fn add_layer(&mut self, index: usize) {
        if self.user_layers.get_mut(index).is_some() {
            warn!("Layer with index '{}' already exists! Skipping.", index);
            return;
        }

        self.user_layers.insert(
            index,
            RenderLayer::new(
                self.config.format,
                Camera::new(Projection::Orthographic {
                    left: 0.0,
                    right: self.config.width as f32,
                    bottom: self.config.height as f32,
                    top: 0.0,
                    z_near: -1.0,
                    z_far: 1.0,
                }),
                self.assets.clone(),
            ),
        );
    }

    // === Retained Rendering ===

    #[inline]
    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshHandle {
        let layer = self.active_layer;
        let handle = self.render_layer_mut(layer).retained.add_mesh(mesh);

        MeshHandle { handle, layer }
    }

    #[inline]
    pub fn get_mesh(&self, handle: MeshHandle) -> &Mesh {
        self.render_layer(handle.layer).retained.get_mesh(*handle)
    }

    #[inline]
    pub fn get_mesh_mut(&mut self, handle: MeshHandle) -> &mut Mesh {
        self.render_layer_mut(handle.layer)
            .retained
            .get_mesh_mut(*handle)
    }

    #[inline]
    pub fn remove_mesh(&mut self, handle: MeshHandle) {
        self.render_layer_mut(handle.layer)
            .retained
            .remove_mesh(*handle);
    }

    #[inline]
    pub fn add_text(&mut self, text: Text) -> TextHandle {
        let layer = self.active_layer;
        let handle = self.render_layer_mut(layer).retained.add_text(text);

        TextHandle { layer, handle }
    }

    #[inline]
    pub fn get_text(&self, handle: TextHandle) -> &Text {
        self.render_layer(handle.layer).retained.get_text(*handle)
    }

    #[inline]
    pub fn get_text_mut(&mut self, handle: TextHandle) -> &mut Text {
        self.render_layer_mut(handle.layer)
            .retained
            .get_text_mut(*handle)
    }

    #[inline]
    pub fn remove_text(&mut self, handle: TextHandle) {
        self.render_layer_mut(handle.layer)
            .retained
            .remove_text(*handle);
    }

    #[inline]
    pub fn add_sprite(&mut self, sprite: Sprite) -> SpriteHandle {
        let layer = self.active_layer;
        let handle = self.render_layer_mut(layer).retained.add_sprite(sprite);

        SpriteHandle { layer, handle }
    }

    #[inline]
    pub fn get_sprite(&self, handle: SpriteHandle) -> &Sprite {
        self.render_layer(handle.layer).retained.get_sprite(*handle)
    }

    #[inline]
    pub fn get_sprite_mut(&mut self, handle: SpriteHandle) -> &mut Sprite {
        self.render_layer_mut(handle.layer)
            .retained
            .get_sprite_mut(*handle)
    }

    #[inline]
    pub fn remove_sprite(&mut self, handle: SpriteHandle) {
        self.render_layer_mut(handle.layer)
            .retained
            .remove_sprite(*handle);
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

    #[inline]
    pub fn draw_point(&mut self, x: f32, y: f32) {
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_point(x, y, color);
    }

    #[inline]
    pub fn draw_point_v<P>(&mut self, p: P)
    where
        P: Into<Vector2>,
    {
        let p = p.into();
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_point(p.x, p.y, color);
    }

    #[inline]
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_line(x1, y1, x2, y2, color);
    }

    #[inline]
    pub fn draw_line_v<P1, P2>(&mut self, p1: P1, p2: P2)
    where
        P1: Into<Vector2>,
        P2: Into<Vector2>,
    {
        let p1 = p1.into();
        let p2 = p2.into();
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_line(p1.x, p1.y, p2.x, p2.y, color);
    }

    /// Draws a filled rectangle in immediate rendering mode.
    #[inline]
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.fill_rect(x, y, w, h, color);
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
        let layer = self.render_layer_mut(self.active_layer);

        layer
            .immediate
            .fill_rect(pos.x, pos.y, size.w(), size.h(), color);
    }

    #[inline]
    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.stroke_rect(x, y, w, h, color);
    }

    #[inline]
    pub fn stroke_rect_v<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<Vector2>,
        S: Into<Size<f32>>,
    {
        let pos = pos.into();
        let size = size.into();
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer
            .immediate
            .stroke_rect(pos.x, pos.y, size.w(), size.h(), color);
    }

    #[inline]
    pub fn draw_image(&mut self, label: Label, x: f32, y: f32) {
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_image(label, x, y, Color::White.into());
    }

    #[inline]
    pub fn draw_image_v<P>(&mut self, label: Label, pos: P)
    where
        P: Into<Vector2>,
    {
        let pos = pos.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer
            .immediate
            .draw_image(label, pos.x, pos.y, Color::White.into());
    }

    #[inline]
    pub fn draw_image_tinted(&mut self, label: Label, x: f32, y: f32) {
        let color: Vector4 = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_image(label, x, y, color);
    }

    #[inline]
    pub fn draw_image_tinted_v<P>(&mut self, label: Label, pos: P)
    where
        P: Into<Vector2>,
    {
        let pos = pos.into();
        let color: Vector4 = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_image(label, pos.x, pos.y, color);
    }

    #[inline]
    pub fn draw_subimage(
        &mut self,
        label: Label,
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
    ) {
        let layer = self.render_layer_mut(self.active_layer);

        layer
            .immediate
            .draw_subimage(label, x, y, sx, sy, sw, sh, Color::White.into());
    }

    #[inline]
    pub fn draw_subimage_v<P, SP, SS>(&mut self, label: Label, pos: P, spos: SP, ssize: SS)
    where
        P: Into<Vector2>,
        SP: Into<Vector2>,
        SS: Into<Size<f32>>,
    {
        let pos = pos.into();
        let spos = spos.into();
        let ssize = ssize.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_subimage(
            label,
            pos.x,
            pos.y,
            spos.x,
            spos.y,
            ssize.width,
            ssize.height,
            Color::White.into(),
        );
    }

    #[inline]
    pub fn draw_subimage_tinted(
        &mut self,
        label: Label,
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
    ) {
        let color: Vector4 = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer
            .immediate
            .draw_subimage(label, x, y, sx, sy, sw, sh, color);
    }

    #[inline]
    pub fn draw_subimage_tinted_v<P, SP, SS>(&mut self, label: Label, pos: P, spos: SP, ssize: SS)
    where
        P: Into<Vector2>,
        SP: Into<Vector2>,
        SS: Into<Size<f32>>,
    {
        let pos = pos.into();
        let spos = spos.into();
        let ssize = ssize.into();
        let color: Vector4 = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_subimage(
            label,
            pos.x,
            pos.y,
            spos.x,
            spos.y,
            ssize.width,
            ssize.height,
            color,
        );
    }

    #[inline]
    pub fn draw_text<T: Into<String>>(&mut self, text: T, x: f32, y: f32) {
        let font_label = self.current_font;
        let text = text.into();
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.draw_text(font_label, &text, x, y, color);
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
        let layer = self.render_layer_mut(self.active_layer);

        layer
            .immediate
            .draw_text(font_label, &text, pos.x, pos.y, color);
    }

    #[inline]
    pub fn debug_text<T: Into<String>>(&mut self, text: T, x: f32, y: f32) {
        let text = text.into();
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer.immediate.debug_text(&text, x, y, color);
    }

    #[inline]
    pub fn debug_text_v<T, P>(&mut self, text: T, pos: P)
    where
        T: AsRef<str>,
        P: Into<Vector2>,
    {
        let pos = pos.into();
        let color = self.draw_color.into();
        let layer = self.render_layer_mut(self.active_layer);

        layer
            .immediate
            .debug_text(text.as_ref(), pos.x, pos.y, color);
    }

    #[inline]
    pub fn debug_logs(&mut self, x: f32) {
        let mut y = 10.0;
        let prev_color = self.draw_color;
        let logs = globals::logs::get();
        let lock = logs.read().expect("Logs lock is poisoned");

        for log in lock.iter() {
            if log.starts_with("[info") {
                self.set_draw_color(Color::Green);
            }

            if log.starts_with("[warn") {
                self.set_draw_color(Color::Yellow);
            }

            if log.starts_with("[error") {
                self.set_draw_color(Color::Red);
            }

            self.debug_text(log, x, y);
            y += 20.0;
        }

        self.set_draw_color(prev_color);
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

        let depth_view = self
            .depth_texture
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_bind_group(0, self.world.camera.view_projection_bind_group(), &[]);
            render_pass.set_bind_group(1, self.assets.bind_group(), &[]);

            self.world.present(
                &mut render_pass,
                &self.retained_pipeline,
                &self.text_pipeline,
            );

            render_pass.set_bind_group(0, self.ui.camera.view_projection_bind_group(), &[]);

            self.ui.present(
                &mut render_pass,
                &self.retained_pipeline,
                &self.text_pipeline,
            );

            for layer in self.user_layers.iter_mut() {
                render_pass.set_bind_group(0, layer.camera.view_projection_bind_group(), &[]);

                layer.present(
                    &mut render_pass,
                    &self.retained_pipeline,
                    &self.text_pipeline,
                );
            }
        }

        gpu.queue().submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}

mod camera;
mod color;
mod immediate;
mod layer;
mod retained;
mod shader;
mod traits;
mod vertex;

use assets::AssetServer;
use logging::info;
use macros::{Get, Set};
use math::Size;
use std::sync::{Arc, OnceLock};
use winit::window::Window;

// === RE-EXPORTS ===

pub use camera::{Camera, Projection};
pub use color::Color;
pub use immediate::Draw;
pub use layer::{Layer, RenderLayer};
pub use retained::{
    Scene, SceneView, Text,
    mesh::{Geometry, Material, Mesh, TextureKind, Transform3d},
};

use crate::shader::Shader;

#[derive(Debug)]
struct Shaders {
    retained: Shader,
    text: Shader,
    immediate: Shader,
    immediate_circle: Shader,
}

static SHADERS: OnceLock<Shaders> = OnceLock::new();

pub(crate) fn retained_shader() -> &'static Shader {
    &SHADERS.get().unwrap().retained
}

pub(crate) fn text_shader() -> &'static Shader {
    &SHADERS.get().unwrap().text
}

pub(crate) fn immediate_shader() -> &'static Shader {
    &SHADERS.get().unwrap().immediate
}

pub(crate) fn immediate_circle_shader() -> &'static Shader {
    &SHADERS.get().unwrap().immediate_circle
}

#[derive(Get, Set)]
pub struct Renderer {
    // Internal stuff
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    clear_color: Color,

    world: RenderLayer,
    ui: RenderLayer,
    user_layers: Vec<RenderLayer>,
    active_layer: Layer,
    /// Cached viewport size
    view: Size<u32>,
}

impl Renderer {
    #[doc(hidden)]
    pub fn new(window: Arc<Window>, assets: &AssetServer) -> Self {
        let retained_shader = Shader::from_wgsl_file(
            include_str!("../../shaders/basic_2d.wgsl"),
            Some("Retained shader"),
        );

        let text_shader =
            Shader::from_wgsl_file(include_str!("../../shaders/text.wgsl"), Some("Text shader"));

        let immediate_shader = Shader::from_wgsl_file(
            include_str!("../../shaders/immediate.wgsl"),
            Some("Immediate shader"),
        );

        let immediate_circle_shader = Shader::from_wgsl_file(
            include_str!("../../shaders/immediate_circle.wgsl"),
            Some("Immediate Circle shader"),
        );

        SHADERS
            .set(Shaders {
                retained: retained_shader,
                text: text_shader,
                immediate: immediate_shader,
                immediate_circle: immediate_circle_shader,
            })
            .unwrap();

        let gpu = gpu::get();
        let view: Size<u32> = window.inner_size().into();

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
            width: view.width,
            height: view.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(gpu.device(), &config);

        let world_camera = Camera::new(Projection::Orthographic {
            left: 0.0,
            right: view.width as f32,
            bottom: view.height as f32,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        });

        let ui_camera = Camera::new(Projection::Orthographic {
            left: 0.0,
            right: view.width as f32,
            bottom: view.height as f32,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        });

        let world = RenderLayer::new(&config, assets, world_camera);
        let ui = RenderLayer::new(&config, assets, ui_camera);

        Self {
            surface,
            config,
            clear_color: Color::rgb(1.0 / 25.0, 1.0 / 25.0, 1.0 / 25.0),
            world,
            ui,
            user_layers: Vec::new(),
            active_layer: Layer::default(),
            view,
        }
    }

    #[inline]
    #[doc(hidden)]
    pub fn resize(&mut self, view: Size<u32>) {
        info!("Resizing viewport to {}x{}", view.width, view.height);

        self.world.queue_resize();
        self.ui.queue_resize();
        self.user_layers.iter_mut().for_each(|l| l.queue_resize());

        self.config.width = view.width;
        self.config.height = view.height;
        self.surface.configure(gpu::device(), &self.config);
        self.view = view;
    }

    #[inline]
    fn layer(&self, id: Layer) -> &RenderLayer {
        match id {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::Custom(i) => &self.user_layers[i],
        }
    }

    #[inline]
    fn layer_mut(&mut self, id: Layer) -> &mut RenderLayer {
        match id {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Custom(i) => &mut self.user_layers[i],
        }
    }

    #[inline]
    #[doc(hidden)]
    pub fn present(&mut self, assets: &AssetServer) {
        let gpu = gpu::get();
        let output = self.surface.get_current_texture().expect("Ouch");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_bind_group(0, self.world.camera.bg(), &[]);
            render_pass.set_bind_group(1, assets.atlas_bg(), &[]);

            self.world.present(self.view, &mut render_pass, assets);

            render_pass.set_bind_group(0, self.ui.camera.bg(), &[]);

            self.ui.present(self.view, &mut render_pass, assets);

            self.user_layers.iter_mut().for_each(|l| {
                render_pass.set_bind_group(0, l.camera.bg(), &[]);
                l.present(self.view, &mut render_pass, assets);
            });
        }

        gpu.queue().submit([encoder.finish()]);
        output.present();
    }
}

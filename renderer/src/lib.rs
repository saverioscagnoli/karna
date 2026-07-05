mod camera;
mod color;
mod immediate;
mod layer;
mod vertex;

use assets::AssetServerGuard;
use gpu::GpuState;
use gpu::PipelineCache;
use math::Size;
use math::Vector4;

pub use crate::camera::Camera;
pub use crate::camera::Projection;
pub use crate::color::Color;
pub use crate::immediate::ImmediateRenderer;
pub use crate::immediate::handle::Draw;
use crate::immediate::imgui::ImguiRenderer;
pub use crate::layer::LayerId;
pub use crate::layer::RenderLayer;
pub use crate::vertex::Vertex;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    cached_size: math::Size<u32>,
    is_surface_configured: bool,
    pipeline_cache: PipelineCache,
    camera_bgl: wgpu::BindGroupLayout,

    pub(crate) clear_color: Vector4<f32>,

    layers: Vec<RenderLayer>,
    active_layer: LayerId,

    pub world: LayerId,
    pub ui: LayerId,
    pub debug: LayerId,

    imgui_renderer: ImguiRenderer,
}

impl Renderer {
    pub fn _create_surface<S: Into<wgpu::SurfaceTarget<'static>>>(
        surface: S,
    ) -> (wgpu::Surface<'static>, wgpu::SurfaceConfiguration) {
        let gpu = GpuState::get();
        let surface = gpu
            .instance
            .create_surface(surface.into())
            .expect("Failed to create surface");

        let caps = surface.get_capabilities(&gpu.adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: 1,
            height: 1,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        (surface, config)
    }

    fn init_pipelines(
        camera_bgl: &wgpu::BindGroupLayout,
        atlas_bgl: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> PipelineCache {
        let mut cache = PipelineCache::new();

        cache.create_pipeline(
            gpu::PipelineDesc {
                shader: "immediate-2d",
                vertex_layout: Vertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::PointList,
            },
            &[camera_bgl, atlas_bgl],
            format,
        );

        cache.create_pipeline(
            gpu::PipelineDesc {
                shader: "immediate-2d",
                vertex_layout: Vertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::LineList,
            },
            &[camera_bgl, atlas_bgl],
            format,
        );

        cache.create_pipeline(
            gpu::PipelineDesc {
                shader: "immediate-2d",
                vertex_layout: Vertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::TriangleList,
            },
            &[&camera_bgl, atlas_bgl],
            format,
        );

        cache
    }

    pub fn _from_surface<'a>(
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        assets: AssetServerGuard<'a>,
        imgui: &mut imgui::Context,
    ) -> Self {
        let camera_bgl = Camera::create_bind_group_layout();

        let mut layers = Vec::new();
        let size = Size::new(config.width, config.height);

        let world = LayerId(layers.len());
        let world_camera = Camera::new(Projection::standard_2d(size), &camera_bgl);
        layers.push(RenderLayer::new(world_camera));

        let ui = LayerId(layers.len());
        let ui_camera = Camera::new(Projection::standard_2d(size), &camera_bgl);
        layers.push(RenderLayer::new(ui_camera));

        let debug = LayerId(layers.len());
        let debug_camera = Camera::new(Projection::standard_2d(size), &camera_bgl);
        layers.push(RenderLayer::new(debug_camera));

        let pipeline_cache = Self::init_pipelines(&camera_bgl, assets.atlas_bgl(), config.format);
        let imgui_renderer = ImguiRenderer::new(imgui, &assets, size, &camera_bgl);

        Self {
            surface,
            config,
            cached_size: size,
            is_surface_configured: false,
            pipeline_cache,
            camera_bgl,
            clear_color: Color::Black.into(),
            layers,
            active_layer: world,
            world,
            ui,
            debug,
            imgui_renderer,
        }
    }

    pub fn add_layer(&mut self, camera_proj: Projection) -> LayerId {
        let camera = Camera::new(camera_proj, &self.camera_bgl);
        let id = LayerId(self.layers.len());

        self.layers.push(RenderLayer::new(camera));

        id
    }

    pub fn layer(&self, id: &LayerId) -> &RenderLayer {
        &self.layers[id.0]
    }

    pub fn layer_mut(&mut self, id: &LayerId) -> &mut RenderLayer {
        &mut self.layers[id.0]
    }

    pub fn active_layer(&self) -> &RenderLayer {
        &self.layers[self.active_layer.0]
    }

    pub fn active_layer_mut(&mut self) -> &mut RenderLayer {
        &mut self.layers[self.active_layer.0]
    }

    pub fn size(&self) -> &math::Size<u32> {
        &self.cached_size
    }

    pub fn _resize(&mut self, width: u32, height: u32) {
        let gpu = GpuState::get();

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&gpu.device, &self.config);
        self.cached_size.width = width;
        self.cached_size.height = height;
        self.is_surface_configured = true;
    }

    pub fn _present<'a>(&'a mut self, assets: &AssetServerGuard<'a>, imgui: &mut imgui::Context) {
        if !self.is_surface_configured {
            return;
        }

        self.imgui_renderer.prepare(imgui, self.cached_size);

        let gpu = GpuState::get();

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                self.surface.configure(&gpu.device, &self.config);
                t
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&gpu.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return,
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Device lost");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color.x as f64,
                            g: self.clear_color.y as f64,
                            b: self.clear_color.z as f64,
                            a: self.clear_color.w as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            for layer in self.layers.iter_mut() {
                rp.set_bind_group(0, &layer.camera.bg, &[]);
                rp.set_bind_group(1, assets.atlas_bg(), &[]);

                layer.present(self.cached_size, &mut rp, &self.pipeline_cache);
            }

            self.imgui_renderer
                .present(&self.pipeline_cache, self.cached_size, &mut rp);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.queue.present(output);
    }
}

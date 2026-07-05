mod camera;
mod color;
mod immediate;
mod layer;
mod vertex;

use assets::AssetServerGuard;
use gpu::GpuState;
use gpu::PipelineCache;
use imgui::Key::S;
use math::Size;
use utils::FastHashMap;

pub use crate::camera::Camera;
pub use crate::camera::Projection;
pub use crate::color::Color;
pub use crate::immediate::ImmediateRenderer;
pub use crate::immediate::handle::Draw;
pub use crate::layer::LayerId;
pub use crate::layer::RenderLayer;
pub use crate::vertex::Vertex;

struct ImguiListRange {
    vtx_base: i32,
    idx_start: u32,
    commands: Vec<imgui::DrawCmdParams>,
    idx_counts: Vec<u32>,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    cached_size: math::Size<u32>,
    is_surface_configured: bool,
    pipeline_cache: PipelineCache,
    camera_bgl: wgpu::BindGroupLayout,

    clear_color: Color,

    layers: Vec<RenderLayer>,
    active_layer: LayerId,

    pub world: LayerId,
    pub ui: LayerId,
    pub debug: LayerId,

    // --- imgui ---
    imgui: imgui::Context,
    imgui_font_texture: gpu::Texture,
    imgui_font_tex_id: imgui::TextureId,
    imgui_custom_textures: FastHashMap<imgui::TextureId, wgpu::BindGroup>,
    imgui_vertex_buf: wgpu::Buffer,
    imgui_index_buf: wgpu::Buffer,
    imgui_vertex_capacity: usize,
    imgui_index_capacity: usize,
    imgui_ranges: Vec<ImguiListRange>,
    imgui_camera: Camera,
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

        // --- imgui setup ---
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let gpu = GpuState::get();

        let imgui_font_tex_id = imgui::TextureId::from(0);
        let imgui_font_texture = {
            let mut fonts = imgui.fonts();
            let atlas_texture = fonts.build_rgba32_texture();
            let font_size = math::Size::new(atlas_texture.width, atlas_texture.height);

            let texture = gpu::Texture::new(
                "imgui font atlas texture",
                font_size,
                atlas_texture.data,
                assets.atlas_bgl(), // reuse existing layout shape — no new pipeline/layout needed
                &gpu.device,
                &gpu.queue,
            );

            fonts.tex_id = imgui_font_tex_id;
            texture
        };

        // Pre-allocate reasonably sized dynamic vertex/index buffers; grown on demand in _present.
        let imgui_vertex_capacity = 4096;
        let imgui_index_capacity = 8192;

        let imgui_vertex_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("imgui vertex buffer"),
            size: (imgui_vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let imgui_index_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("imgui index buffer"),
            size: (imgui_index_capacity * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let imgui_camera = Camera::new(Projection::standard_2d(size), &camera_bgl);

        Self {
            surface,
            config,
            cached_size: size,
            is_surface_configured: false,
            pipeline_cache,
            camera_bgl,
            clear_color: Color::Black,
            layers,
            active_layer: world,
            world,
            ui,
            debug,

            imgui,
            imgui_font_texture,
            imgui_font_tex_id,
            imgui_custom_textures: FastHashMap::default(),
            imgui_vertex_buf,
            imgui_index_buf,
            imgui_vertex_capacity,
            imgui_index_capacity,
            imgui_ranges: Vec::new(),
            imgui_camera,
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

    pub fn imgui(&mut self) -> &mut imgui::Context {
        &mut self.imgui
    }

    pub fn _present<'a>(&'a mut self, assets: &AssetServerGuard<'a>) {
        if !self.is_surface_configured {
            return;
        }

        self.prepare_imgui();

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

        // Take disjoint field references up front, before the render pass borrows self.layers.
        let Renderer {
            layers,
            pipeline_cache,
            imgui_camera,
            imgui_font_texture,
            imgui_font_tex_id,
            imgui_custom_textures,
            imgui_vertex_buf,
            imgui_index_buf,
            imgui_ranges,
            cached_size,
            config,
            clear_color,
            ..
        } = self;

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear((*clear_color).into()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            for layer in layers.iter_mut() {
                rp.set_bind_group(0, &layer.camera.bg, &[]);
                rp.set_bind_group(1, assets.atlas_bg(), &[]);

                layer.present(
                    Size::new(config.width, config.height),
                    &mut rp,
                    pipeline_cache,
                );
            }

            Self::draw_imgui_pass(
                pipeline_cache,
                &imgui_camera.bg,
                &imgui_font_texture.bind_group,
                *imgui_font_tex_id,
                imgui_custom_textures,
                imgui_vertex_buf,
                imgui_index_buf,
                imgui_ranges,
                *cached_size,
                &mut rp,
            );
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.queue.present(output);
    }

    /// PREPARE phase: converts imgui's draw data into your `Vertex` type,
    /// grows/uploads GPU buffers as needed, and stashes per-draw-list ranges
    /// for the DRAW phase. Requires `&mut self`. Must be called *before* the
    /// render pass is created (it needs exclusive access to `self`).
    fn prepare_imgui(&mut self) {
        let draw_data = self.imgui.render(); // borrow starts and ends within this function

        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if fb_width <= 0.0 || fb_height <= 0.0 {
            self.imgui_ranges.clear();
            return;
        }

        let mut all_vertices: Vec<Vertex> = Vec::new();
        let mut all_indices: Vec<u16> = Vec::new();
        let mut ranges: Vec<ImguiListRange> = Vec::new();

        for draw_list in draw_data.draw_lists() {
            let vtx_base = all_vertices.len() as i32;
            let idx_start = all_indices.len() as u32;

            all_vertices.extend(draw_list.vtx_buffer().iter().map(Vertex::from));
            all_indices.extend_from_slice(draw_list.idx_buffer());

            let mut commands = Vec::new();
            let mut idx_counts = Vec::new();
            for cmd in draw_list.commands() {
                if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                    commands.push(cmd_params);
                    idx_counts.push(count as u32);
                }
            }

            ranges.push(ImguiListRange {
                vtx_base,
                idx_start,
                commands,
                idx_counts,
            });
        }

        if all_vertices.is_empty() {
            self.imgui_ranges.clear();
            return;
        }

        let gpu = GpuState::get();

        if all_vertices.len() > self.imgui_vertex_capacity {
            self.imgui_vertex_capacity = all_vertices.len().next_power_of_two();
            self.imgui_vertex_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("imgui vertex buffer"),
                size: (self.imgui_vertex_capacity * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if all_indices.len() > self.imgui_index_capacity {
            self.imgui_index_capacity = all_indices.len().next_power_of_two();
            self.imgui_index_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("imgui index buffer"),
                size: (self.imgui_index_capacity * std::mem::size_of::<u16>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        write_buffer_aligned(
            &gpu.queue,
            &self.imgui_vertex_buf,
            utils::as_u8_slice(&all_vertices),
        );
        write_buffer_aligned(
            &gpu.queue,
            &self.imgui_index_buf,
            utils::as_u8_slice(&all_indices),
        );

        self.imgui_camera.update(self.cached_size);

        self.imgui_ranges = ranges;
    }

    fn draw_imgui_pass<'a>(
        pipeline_cache: &'a PipelineCache,
        imgui_camera_bg: &'a wgpu::BindGroup,
        imgui_font_bind_group: &'a wgpu::BindGroup,
        imgui_font_tex_id: imgui::TextureId,
        imgui_custom_textures: &'a FastHashMap<imgui::TextureId, wgpu::BindGroup>,
        imgui_vertex_buf: &'a wgpu::Buffer,
        imgui_index_buf: &'a wgpu::Buffer,
        imgui_ranges: &'a [ImguiListRange],
        fb_size: math::Size<u32>,
        rp: &mut wgpu::RenderPass<'a>,
    ) {
        if imgui_ranges.is_empty() {
            return;
        }

        let pipeline = pipeline_cache.get_pipeline(&gpu::PipelineDesc {
            shader: "immediate-2d",
            vertex_layout: Vertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
        });
        rp.set_pipeline(pipeline);
        rp.set_vertex_buffer(0, imgui_vertex_buf.slice(..));
        rp.set_index_buffer(imgui_index_buf.slice(..), wgpu::IndexFormat::Uint16);
        rp.set_bind_group(0, imgui_camera_bg, &[]);

        for range in imgui_ranges {
            let mut idx_cursor = range.idx_start;
            for (cmd_params, count) in range.commands.iter().zip(range.idx_counts.iter()) {
                let clip = cmd_params.clip_rect;
                let x = clip[0].max(0.0) as u32;
                let y = clip[1].max(0.0) as u32;
                let w = ((clip[2] - clip[0]).max(0.0) as u32).min(fb_size.width.saturating_sub(x));
                let h = ((clip[3] - clip[1]).max(0.0) as u32).min(fb_size.height.saturating_sub(y));

                if w == 0 || h == 0 {
                    idx_cursor += count;
                    continue;
                }
                rp.set_scissor_rect(x, y, w, h);

                let bind_group = if cmd_params.texture_id == imgui_font_tex_id {
                    imgui_font_bind_group
                } else {
                    imgui_custom_textures
                        .get(&cmd_params.texture_id)
                        .expect("Unregistered imgui texture id")
                };
                rp.set_bind_group(1, bind_group, &[]);

                rp.draw_indexed(idx_cursor..(idx_cursor + count), range.vtx_base, 0..1);
                idx_cursor += count;
            }
        }
    }
}

fn write_buffer_aligned(queue: &wgpu::Queue, buffer: &wgpu::Buffer, data: &[u8]) {
    const ALIGNMENT: usize = wgpu::COPY_BUFFER_ALIGNMENT as usize; // typically 4
    let padded_len = (data.len() + ALIGNMENT - 1) / ALIGNMENT * ALIGNMENT;

    if padded_len == data.len() {
        queue.write_buffer(buffer, 0, data);
    } else {
        let mut padded = Vec::with_capacity(padded_len);
        padded.extend_from_slice(data);
        padded.resize(padded_len, 0); // pad with zero bytes
        queue.write_buffer(buffer, 0, &padded);
    }
}

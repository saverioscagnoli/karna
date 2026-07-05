use assets::AssetServerGuard;
use gpu::GpuState;
use gpu::PipelineCache;
use utils::FastHashMap;

use crate::Camera;
use crate::Projection;
use crate::Vertex;

struct ImguiListRange {
    vertex_base: i32,
    index_start: u32,
    index_counts: Vec<u32>,
    commands: Vec<imgui::DrawCmdParams>,
}

pub struct ImguiRenderer {
    font_texture: gpu::Texture,
    font_texture_id: imgui::TextureId,
    custom_textures: FastHashMap<imgui::TextureId, wgpu::BindGroup>,
    vertex_buffer: gpu::Buffer<Vertex>,
    index_buffer: gpu::Buffer<u16>,
    ranges: Vec<ImguiListRange>,
    camera: Camera,
}

impl ImguiRenderer {
    pub fn new<'a>(
        ctx: &mut imgui::Context,
        assets: &AssetServerGuard<'a>,
        view: math::Size<u32>,
        camera_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let gpu = GpuState::get();
        let font_texture_id = imgui::TextureId::from(0);
        let font_texture = {
            let fonts = ctx.fonts();
            let atlas_texture = fonts.build_rgba32_texture();
            let font_size = math::Size::new(atlas_texture.width, atlas_texture.height);

            let texture = gpu::Texture::new(
                "imgui font atlas texture",
                font_size,
                atlas_texture.data,
                assets.atlas_bgl(),
                &gpu.device,
                &gpu.queue,
            );

            fonts.tex_id = font_texture_id;
            texture
        };

        let vertex_buffer = gpu::Buffer::new_with_capacity(
            "imgui vertex buffer",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            4096,
        );

        let index_buffer = gpu::Buffer::new_with_capacity(
            "imgui vertex buffer",
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            8192,
        );

        let camera = Camera::new(Projection::standard_2d(view), camera_bgl);

        Self {
            font_texture,
            font_texture_id,
            custom_textures: FastHashMap::default(),
            vertex_buffer,
            index_buffer,
            ranges: Vec::new(),
            camera,
        }
    }

    pub fn prepare(&mut self, ctx: &mut imgui::Context, size: math::Size<u32>) {
        let draw_data = ctx.render();

        let fb_w = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_h = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

        if fb_w <= 0.0 || fb_h <= 0.0 {
            self.ranges.clear();
            return;
        }

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut ranges: Vec<ImguiListRange> = Vec::new();

        for list in draw_data.draw_lists() {
            let vertex_base = vertices.len() as i32;
            let index_start = indices.len() as u32;

            vertices.extend(list.vtx_buffer().iter().map(Vertex::from));
            indices.extend_from_slice(list.idx_buffer());

            let mut commands = Vec::new();
            let mut index_counts = Vec::new();

            for cmd in list.commands() {
                if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                    commands.push(cmd_params);
                    index_counts.push(count as u32);
                }
            }

            ranges.push(ImguiListRange {
                vertex_base,
                index_start,
                index_counts,
                commands,
            });
        }

        if vertices.is_empty() {
            self.ranges.clear();
            return;
        }

        // Align indices to prevend alignment errors
        if indices.len() % 2 != 0 {
            indices.push(0);
        }

        self.vertex_buffer.write(0, &vertices);
        self.index_buffer.write(0, &indices);
        self.camera.update(size);
        self.ranges = ranges;
    }

    pub fn present<'a>(
        &'a self,
        pipeline_cache: &'a PipelineCache,
        size: math::Size<u32>,
        rp: &mut wgpu::RenderPass<'a>,
    ) {
        if self.ranges.is_empty() {
            return;
        }

        let pipeline = pipeline_cache.get_pipeline(&gpu::PipelineDesc {
            shader: "immediate-2d",
            vertex_layout: Vertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
        });

        rp.set_pipeline(pipeline);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rp.set_bind_group(0, &self.camera.bg, &[]);

        for range in &self.ranges {
            let mut cursor = range.index_start;

            for (params, count) in range.commands.iter().zip(&range.index_counts) {
                let clip = params.clip_rect;
                let x = clip[0].max(0.0) as u32;
                let y = clip[1].max(0.0) as u32;
                let w = ((clip[2] - clip[0]).max(0.0) as u32).min(size.width.saturating_sub(x));
                let h = ((clip[3] - clip[1]).max(0.0) as u32).min(size.height.saturating_sub(y));

                if w == 0 || h == 0 {
                    cursor += count;
                    continue;
                }

                rp.set_scissor_rect(x, y, w, h);

                let bg = if params.texture_id == self.font_texture_id {
                    &self.font_texture.bind_group
                } else {
                    self.custom_textures
                        .get(&params.texture_id)
                        .expect("unregistered imgui texture")
                };

                rp.set_bind_group(1, bg, &[]);

                rp.draw_indexed(cursor..cursor + count, range.vertex_base, 0..1);
                cursor += count;
            }
        }
    }
}

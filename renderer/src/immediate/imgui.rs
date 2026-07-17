use assets::AssetsRead;
use gpu::GpuState;
use imgui::ImguiCmd;

use crate::Camera;
use crate::Projection;
use crate::camera::CameraData;
use crate::immediate::batcher::Batcher;
use crate::layouts;
use crate::layouts::LayoutDesc;
use crate::vertex::MODE_TEXTURED;
use crate::vertex::ShapeVertex;

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct ImguiPacket {
    pub vertices: Vec<ShapeVertex>,
    pub indices: Vec<u32>,
    pub cmds: Vec<ImguiCmd>,
    pub display_pos: [f32; 2],
    pub display_size: [f32; 2],
    pub fb_scale: [f32; 2],
}

impl ImguiPacket {
    pub fn record(&mut self, dd: &imgui::DrawData, font_uv: math::Vector4<f32>) {
        self.vertices.clear();
        self.indices.clear();
        self.cmds.clear();

        self.display_pos = dd.display_pos;
        self.display_size = dd.display_size;
        self.fb_scale = dd.framebuffer_scale;

        if dd.total_vtx_count() == 0 {
            return;
        }

        let (ox, oy, ew, eh) = (font_uv.x, font_uv.y, font_uv.z, font_uv.w);

        let font_rect = math::Vector4::new(ox, oy, ox + ew, oy + eh);

        let z2 = math::Vector2::new(0.0, 0.0);
        let z4 = math::Vector4::new(0.0, 0.0, 0.0, 0.0);

        for list in dd.draw_lists() {
            let vtx_base = self.vertices.len() as i32;
            let idx_base = self.indices.len() as u32;

            self.vertices.extend(list.vtx_buffer().iter().map(|v| {
                let uv = math::Vector2::new(ox + v.uv[0] * ew, oy + v.uv[1] * eh);
                let col = math::Vector4::new(
                    v.rgba()[0] as f32 / 255.0,
                    v.rgba()[1] as f32 / 255.0,
                    v.rgba()[2] as f32 / 255.0,
                    v.rgba()[3] as f32 / 255.0,
                );

                ShapeVertex {
                    position: math::Vector3::new(v.pos[0], v.pos[1], 0.0),
                    color: col,
                    uv,
                    local: z2,
                    params: z4,
                    uv_rect: font_rect,
                    mode: MODE_TEXTURED,
                }
            }));

            self.indices
                .extend(list.idx_buffer().iter().map(|&i| i as u32)); // Uint32, like the Batcher

            for cmd in list.commands() {
                if let imgui::DrawCmd::Elements {
                    count, cmd_params, ..
                } = cmd
                {
                    self.cmds.push(ImguiCmd {
                        clip: cmd_params.clip_rect,
                        vtx_offset: vtx_base + cmd_params.vtx_offset as i32,
                        idx_offset: idx_base + cmd_params.idx_offset as u32,
                        count: count as u32,
                    });
                }
            }
        }
    }
}

pub struct ImguiRenderer {
    camera: Camera,
    camera_size: math::Size<u32>,
    camera_buffer: gpu::Buffer<CameraData>,
    camera_bg: wgpu::BindGroup,
    batcher: Batcher<ShapeVertex>,
}

impl ImguiRenderer {
    pub(crate) fn new(gpu: &GpuState) -> Self {
        let camera_buffer = gpu::Buffer::new_with_capacity(
            "imgui camera uniform buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            1,
        );

        let camera_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("imgui camera bind group"),
            layout: layouts::camera(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.wgpu().as_entire_binding(),
            }],
        });

        let size = math::Size::new(1, 1);

        Self {
            camera: Camera::new(Projection::standard_2d(size)),
            camera_size: size,
            camera_buffer,
            camera_bg,
            batcher: Batcher::new(),
        }
    }

    pub(crate) fn present<'rp>(
        &'rp mut self,
        packet: &ImguiPacket,
        viewport: math::Size<u32>,
        pass: &mut wgpu::RenderPass<'rp>,
        pipelines: &gpu::PipelineCache,
        format: wgpu::TextureFormat,
        assets: AssetsRead<'rp>,
    ) {
        if packet.cmds.is_empty() {
            return;
        }

        if self.camera_size != viewport {
            self.camera.update(viewport);
            self.camera_size = viewport;
        }

        self.camera_buffer.write(0, &[self.camera.data()]);
        self.batcher.upload(&packet.vertices, &packet.indices);

        let desc = gpu::PipelineDesc {
            shader: "immediate-2d",
            vertex_layout: ShapeVertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
            instance_layout: None,
            depth: gpu::DepthMode::Disabled,
            cull: None,
        };

        let pipeline = pipelines.get_or_create(desc, format, &layouts::immediate());

        pass.set_bind_group(0, &self.camera_bg, &[]);
        pass.set_bind_group(1, assets.atlas_bg(), &[]);

        self.batcher.bind(pass, Some(&pipeline));

        let (fw, fh) = (viewport.width, viewport.height);
        let [dx, dy] = packet.display_pos;
        let [sx, sy] = packet.fb_scale;

        for cmd in &packet.cmds {
            let x1 = ((cmd.clip[0] - dx) * sx).max(0.0).floor() as u32;
            let y1 = ((cmd.clip[1] - dy) * sy).max(0.0).floor() as u32;
            let x2 = (((cmd.clip[2] - dx) * sx).max(0.0).ceil() as u32).min(fw);
            let y2 = (((cmd.clip[3] - dy) * sy).max(0.0).ceil() as u32).min(fh);

            if x2 <= x1 || y2 <= y1 {
                continue;
            }

            pass.set_scissor_rect(x1, y1, x2 - x1, y2 - y1);
            pass.draw_indexed(
                cmd.idx_offset..cmd.idx_offset + cmd.count,
                cmd.vtx_offset,
                0..1,
            );
        }

        pass.set_scissor_rect(0, 0, fw, fh);
    }
}

use assets::AssetsRead;
use gpu::GpuState;
use gpu::Vertex;
use imgui::ImguiPacket;

use crate::Camera;
use crate::Layouts;
use crate::Projection;
use crate::camera::CameraData;
use crate::immediate::batcher::Batcher;

pub struct ImguiRenderer {
    camera: Camera,
    camera_size: math::Size<u32>,
    camera_buffer: gpu::Buffer<CameraData>,
    camera_bg: wgpu::BindGroup,
    batcher: Batcher<Vertex>,
}

impl ImguiRenderer {
    pub(crate) fn new(gpu: &GpuState, camera_layout: &wgpu::BindGroupLayout) -> Self {
        let camera_buffer = gpu::Buffer::new_with_capacity(
            "imgui camera uniform buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            1,
        );

        let camera_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("imgui camera bind group"),
            layout: camera_layout,
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
        layouts: &Layouts,
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
            vertex_layout: Vertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
        };

        let pipeline = pipelines.get_or_create(desc, format, &layouts.as_array());

        pass.set_bind_group(0, &self.camera_bg, &[]);
        pass.set_bind_group(1, assets.atlas_bg(), &[]);

        self.batcher.bind(pass, &pipeline);

        let (fw, fh) = (viewport.width, viewport.height);
        let [dx, dy] = packet.display_pos;
        let [sx, sy] = packet.fb_scale;

        for cmd in &packet.cmds {
            let x1 = ((cmd.clip[0] - dx) * sx).max(0.0).floor() as u32;
            let y1 = ((cmd.clip[1] - dy) * sy).max(0.0).floor() as u32;
            let x2 = (((cmd.clip[2] - dx) * sx).max(0.0).ceil() as u32).min(fw);
            let y2 = (((cmd.clip[3] - dy) * sy).max(0.0).ceil() as u32).min(fh);

            // wgpu validation-errors on zero-area or out-of-bounds scissors
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

        // Scissor state persists for the rest of the pass — reset it.
        pass.set_scissor_rect(0, 0, fw, fh);
    }
}

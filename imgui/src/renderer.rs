use std::slice;

use dear_imgui_sys::*;
use gpu::Gpu;
use sdl3::{rect::Rect, video::Window};
use utils::{FastHashMap, WindowId};

use crate::textures::ImguiTextureRegistry;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImguiVertex {
    pub position: math::Vector2<f32>,
    pub uv: math::Vector2<f32>,
    pub color: math::Vector4<u8>,
}

impl gpu::LayoutDesc for ImguiVertex {
    const ATTRIBUTES: &'static [gpu::VertexAttribute] = &[
        gpu::VertexAttribute {
            location: 0,
            format: gpu::VertexElementFormat::Float2,
            offset: 0,
        },
        gpu::VertexAttribute {
            location: 1,
            format: gpu::VertexElementFormat::Float2,
            offset: 8,
        },
        gpu::VertexAttribute {
            location: 2,
            format: gpu::VertexElementFormat::Ubyte4Norm,
            offset: 16,
        },
    ];
}

#[derive(Debug, Clone, Copy)]
struct DrawCall {
    texture: ImTextureID,
    clip: Option<Rect>,
    elem_count: u32,
    idx_offset: u32,
    vtx_offset: i32,
}

struct ImguiTarget {
    vertex_buffer: gpu::Buffer<ImguiVertex>,
    index_buffer: gpu::Buffer<ImDrawIdx>,
}

impl ImguiTarget {
    fn new(gpu: &Gpu, id: WindowId) -> Self {
        Self {
            vertex_buffer: gpu::Buffer::new(
                gpu,
                format!("{id:?}-imgui-vertex-buffer"),
                gpu::BufferUsages::VERTEX,
                4096,
            ),
            index_buffer: gpu::Buffer::new(
                gpu,
                format!("{id:?}-imgui-index-buffer"),
                gpu::BufferUsages::INDEX,
                8192,
            ),
        }
    }
}

pub struct ImguiRenderer {
    targets: FastHashMap<WindowId, ImguiTarget>,
    textures: ImguiTextureRegistry,
    vertices: Vec<ImguiVertex>,
    indices: Vec<ImDrawIdx>,
    calls: Vec<DrawCall>,
    projection: math::Matrix4<f32>,
}

impl ImguiRenderer {
    pub fn new() -> Self {
        Self {
            targets: FastHashMap::default(),
            textures: ImguiTextureRegistry::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            calls: Vec::new(),
            projection: math::Matrix4::identity(),
        }
    }

    unsafe fn flatten(&mut self, draw_data: *mut ImDrawData) {
        unsafe {
            let pos = (*draw_data).DisplayPos;
            let size = (*draw_data).DisplaySize;
            let scale = (*draw_data).FramebufferScale;

            self.projection = math::Matrix4::orthographic(
                pos.x,
                pos.x + size.x,
                pos.y + size.y,
                pos.y,
                -1.0,
                1.0,
            );

            let fb_width = size.x * scale.x;
            let fb_height = size.y * scale.y;

            let lists = &(*draw_data).CmdLists;

            for n in 0..(*draw_data).CmdListsCount {
                let list = *lists.Data.offset(n as isize);

                if list.is_null() {
                    continue;
                }

                let vtx_base = self.vertices.len() as i32;
                let idx_base = self.indices.len() as u32;

                let vtx = &(*list).VtxBuffer;
                let idx = &(*list).IdxBuffer;

                self.vertices.extend_from_slice(slice::from_raw_parts(
                    vtx.Data as *const ImguiVertex,
                    vtx.Size as usize,
                ));

                self.indices
                    .extend_from_slice(slice::from_raw_parts(idx.Data, idx.Size as usize));

                let cmds = &(*list).CmdBuffer;

                for c in 0..cmds.Size {
                    let cmd = cmds.Data.offset(c as isize);

                    if (*cmd).UserCallback.is_some() {
                        continue;
                    }

                    if (*cmd).ElemCount == 0 {
                        continue;
                    }

                    self.calls.push(DrawCall {
                        texture: ImDrawCmd_GetTexID(cmd),
                        clip: scissor(&(*cmd).ClipRect, pos, scale, fb_width, fb_height),
                        elem_count: (*cmd).ElemCount,
                        idx_offset: idx_base + (*cmd).IdxOffset,
                        vtx_offset: vtx_base + (*cmd).VtxOffset as i32,
                    });
                }
            }
        }
    }

    pub fn upload(
        &mut self,
        gpu: &Gpu,
        copy_pass: &gpu::CopyPass,
        window: &Window,
        draw_data: crate::DrawData,
    ) {
        self.vertices.clear();
        self.indices.clear();
        self.calls.clear();

        if draw_data.is_empty() {
            return;
        }

        let draw_data = draw_data.as_ptr();

        self.textures.update(gpu, draw_data);

        unsafe {
            if !(*draw_data).Valid || (*draw_data).CmdListsCount == 0 {
                return;
            }

            self.flatten(draw_data);
        }

        let target = self
            .targets
            .entry(window.id())
            .or_insert_with(|| ImguiTarget::new(gpu, window.id()));

        #[rustfmt::skip]
        target.vertex_buffer.write_all(gpu, copy_pass, &self.vertices);
        target.index_buffer.write_all(gpu, copy_pass, &self.indices);
    }

    pub fn record(
        &self,
        gpu: &Gpu,
        pip: &gpu::RenderPipeline,
        cmd: &gpu::CommandBuffer,
        rpass: &gpu::RenderPass,
        window: &Window,
    ) {
        if self.calls.is_empty() {
            return;
        }

        let Some(target) = self.targets.get(&window.id()) else {
            return;
        };

        rpass.bind_graphics_pipeline(pip);

        cmd.push_vertex_uniform_data(0, &self.projection);

        rpass.bind_vertex_buffers(0, &[target.vertex_buffer.binding()]);
        rpass.bind_index_buffer(
            &target.index_buffer.binding(),
            gpu::IndexElementSize::_16BIT,
        );

        let (w, h) = window.size_in_pixels();
        let full = Rect::new(0, 0, w, h);

        for call in &self.calls {
            let Some(texture) = self.textures.get(call.texture) else {
                continue;
            };

            let Some(clip) = call.clip else {
                continue;
            };

            rpass.set_scissor(clip);
            rpass.bind_fragment_samplers(0, &[texture.binding(gpu)]);
            rpass.draw_indexed_primitives(call.elem_count, 1, call.idx_offset, call.vtx_offset, 0);

            rpass.set_scissor(full);
        }
    }
}

fn scissor(
    clip: &ImVec4_c,
    display_pos: ImVec2_c,
    scale: ImVec2_c,
    fb_width: f32,
    fb_height: f32,
) -> Option<Rect> {
    let min_x = ((clip.x - display_pos.x) * scale.x).max(0.0);
    let min_y = ((clip.y - display_pos.y) * scale.y).max(0.0);
    let max_x = ((clip.z - display_pos.x) * scale.x).min(fb_width);
    let max_y = ((clip.w - display_pos.y) * scale.y).min(fb_height);

    if max_x <= min_x || max_y <= min_y {
        return None;
    }

    Some(Rect::new(
        min_x as i32,
        min_y as i32,
        (max_x - min_x) as u32,
        (max_y - min_y) as u32,
    ))
}

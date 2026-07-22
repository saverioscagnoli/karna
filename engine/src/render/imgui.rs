use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use logging::warn;

use crate::render::vertex::LayoutDesc;

/// Ids handed out to imgui textures. Global so every window's context gets
/// unique ids even though each renderer keeps its own texture map.
/// Id 0 is reserved: imgui uses it as "no texture".
pub(crate) static NEXT_TEXTURE_ID: AtomicU64 = AtomicU64::new(1);

/// Layout-identical to `imgui::DrawVert`, so draw lists can be copied
/// straight into the packet without per-vertex conversion.
#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct ImguiVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    /// Packed sRGB RGBA bytes, exactly as imgui produces them.
    /// The imgui vertex shader linearizes them.
    pub color: u32,
}

const _: () = assert!(
    std::mem::size_of::<ImguiVertex>() == std::mem::size_of::<imgui::DrawVert>()
);

impl LayoutDesc for ImguiVertex {
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

/// One imgui draw command: a scissored, textured range of the index buffer.
#[derive(Debug, Clone)]
pub struct ImguiBatch {
    /// Scissor rectangle in framebuffer pixels (left, top, right, bottom).
    pub clip: [f32; 4],
    pub texture: u64,
    pub first_index: u32,
    pub index_count: u32,
    pub vertex_offset: i32,
}

/// A GPU texture operation requested through imgui's 1.92 texture protocol
/// (`ImTextureData`). Pixels are always RGBA8.
#[derive(Debug, Clone)]
pub enum ImguiTextureUpdate {
    Create {
        id: u64,
        size: math::Size<u32>,
        pixels: Vec<u8>,
    },
    Write {
        id: u64,
        origin: math::Vector2<u32>,
        size: math::Size<u32>,
        pixels: Vec<u8>,
    },
    Destroy {
        id: u64,
    },
}

/// CPU snapshot of one imgui frame, produced on the window thread and
/// consumed by the main-thread renderer alongside the regular layers.
#[derive(Default)]
#[derive(Debug, Clone)]
pub struct ImguiPacket {
    pub vertices: Vec<ImguiVertex>,
    pub indices: Vec<u16>,
    pub batches: Vec<ImguiBatch>,
    /// Texture ops to run before drawing. Imgui is only told once about each
    /// request, so these must never be dropped: when a frame is replaced
    /// unrendered, the window thread carries them over to the next one.
    pub textures: Vec<ImguiTextureUpdate>,
    pub display_pos: [f32; 2],
    pub display_size: [f32; 2],
    pub framebuffer_scale: [f32; 2],
}

impl ImguiPacket {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
        self.textures.clear();
    }

    /// Snapshots rendered draw data: services texture requests, then copies
    /// geometry and draw commands. Must run on the thread owning the context.
    pub fn capture(&mut self, draw_data: &mut imgui::DrawData) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
        // Texture ops accumulate: the vec may hold updates carried over from
        // a frame the main thread never rendered.

        self.display_pos = draw_data.display_pos();
        self.display_size = draw_data.display_size();
        self.framebuffer_scale = draw_data.framebuffer_scale();

        // Textures first: draw commands resolve their texture id through
        // `ImDrawCmd_GetTexID`, which reads the id assigned here.
        self.capture_textures(draw_data);

        let pos = self.display_pos;
        let scale = self.framebuffer_scale;

        for list in draw_data.draw_lists() {
            let vtx_base = self.vertices.len() as i32;
            let idx_base = self.indices.len() as u32;

            let verts = list.vtx_buffer();
            // Safety: ImguiVertex is layout-identical to DrawVert (asserted above).
            let verts = unsafe {
                std::slice::from_raw_parts(verts.as_ptr().cast::<ImguiVertex>(), verts.len())
            };

            self.vertices.extend_from_slice(verts);
            self.indices.extend_from_slice(list.idx_buffer());

            for cmd in list.commands() {
                match cmd {
                    imgui::DrawCmd::Elements {
                        count,
                        cmd_params,
                        raw_cmd,
                    } => {
                        // With the modern texture system the raw TexID field
                        // can be 0; the effective id lives behind the command.
                        let texture =
                            unsafe { imgui::sys::ImDrawCmd_GetTexID(raw_cmd as *mut _) };

                        let c = cmd_params.clip_rect;
                        let clip = [
                            (c[0] - pos[0]) * scale[0],
                            (c[1] - pos[1]) * scale[1],
                            (c[2] - pos[0]) * scale[0],
                            (c[3] - pos[1]) * scale[1],
                        ];

                        if count == 0 || clip[2] <= clip[0] || clip[3] <= clip[1] {
                            continue;
                        }

                        self.batches.push(ImguiBatch {
                            clip,
                            texture: texture as u64,
                            first_index: idx_base + cmd_params.idx_offset as u32,
                            index_count: count as u32,
                            vertex_offset: vtx_base + cmd_params.vtx_offset as i32,
                        });
                    }

                    imgui::DrawCmd::ResetRenderState
                    | imgui::DrawCmd::SetSamplerLinear
                    | imgui::DrawCmd::SetSamplerNearest => {}

                    imgui::DrawCmd::RawCallback { .. } => {
                        warn!("Imgui raw draw callbacks are not supported");
                    }
                }
            }
        }
    }

    /// Services imgui texture requests (create/update/destroy), recording the
    /// pixel uploads for the renderer and acknowledging them to imgui.
    fn capture_textures(&mut self, draw_data: &mut imgui::DrawData) {
        let mut cursor = draw_data.textures_mut();

        while let Some(mut tex) = cursor.next() {
            match tex.status() {
                imgui::TextureStatus::WantCreate => {
                    // Recreations (e.g. font atlas resize) keep their id so
                    // the renderer replaces the texture in place.
                    let id = match tex.tex_id().id() {
                        0 => NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed),
                        id => id,
                    };

                    let size = math::Size::new(tex.width(), tex.height());

                    let Some(pixels) = rgba_pixels(&tex, 0, 0, size.width, size.height) else {
                        warn!("Imgui texture {} has no pixels, skipping create", id);
                        continue;
                    };

                    self.textures
                        .push(ImguiTextureUpdate::Create { id, size, pixels });

                    tex.set_tex_id(imgui::TextureId::new(id));
                    tex.set_status(imgui::TextureStatus::OK);
                }

                imgui::TextureStatus::WantUpdates => {
                    let id = tex.tex_id().id();
                    let rects: Vec<_> = tex.updates().collect();

                    for r in rects {
                        let (x, y) = (r.x as u32, r.y as u32);
                        let (w, h) = (r.w as u32, r.h as u32);

                        let Some(pixels) = rgba_pixels(&tex, x, y, w, h) else {
                            continue;
                        };

                        self.textures.push(ImguiTextureUpdate::Write {
                            id,
                            origin: math::Vector2::new(x, y),
                            size: math::Size::new(w, h),
                            pixels,
                        });
                    }

                    tex.set_status(imgui::TextureStatus::OK);
                }

                imgui::TextureStatus::WantDestroy => {
                    let id = tex.tex_id().id();

                    if id != 0 {
                        self.textures.push(ImguiTextureUpdate::Destroy { id });
                    }

                    // Also clears the TexID on the imgui side.
                    tex.set_status(imgui::TextureStatus::Destroyed);
                }

                imgui::TextureStatus::OK | imgui::TextureStatus::Destroyed => {}
            }
        }
    }
}

/// Copies a sub-rectangle of an imgui texture as tightly-packed RGBA8.
fn rgba_pixels(tex: &imgui::TextureData, x: u32, y: u32, w: u32, h: u32) -> Option<Vec<u8>> {
    let bpp = tex.bytes_per_pixel();
    let row_len = w as usize * bpp;
    let mut out = Vec::with_capacity(w as usize * h as usize * 4);

    for row in y..y + h {
        // `pixels_at` returns the buffer from (x, row) to the end.
        let bytes = tex.pixels_at(x, row)?;
        let bytes = bytes.get(..row_len)?;

        match tex.format() {
            imgui::TextureFormat::RGBA32 => out.extend_from_slice(bytes),
            imgui::TextureFormat::Alpha8 => {
                for &a in bytes {
                    out.extend_from_slice(&[255, 255, 255, a]);
                }
            }
        }
    }

    Some(out)
}

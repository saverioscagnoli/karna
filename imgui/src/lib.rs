mod input;

use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use gpu::Vertex;
pub use imgui::*;
use parking_lot::Mutex;
use parking_lot::MutexGuard;
use utils::FastHashMap;
use winit::window::WindowId;

pub use crate::input::winit_keycode_to_imgui;
pub use crate::input::winit_mousebutton_to_imgui;

#[derive(Debug, Clone, Copy)]
pub enum ImguiError {
    NotRegistered(WindowId),
    ForeignContextActive,
}

impl fmt::Display for ImguiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(id) => write!(f, "no imgui context registered for {id:?}"),
            Self::ForeignContextActive => write!(
                f,
                "another imgui context outside this manager is currently active"
            ),
        }
    }
}

impl std::error::Error for ImguiError {}

#[derive(Default)]
pub struct ImguiManager {
    contexts: FastHashMap<WindowId, Option<imgui::SuspendedContext>>,
}

impl ImguiManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_window(&mut self, id: WindowId, size: math::Size<f32>) {
        let mut ctx = imgui::Context::create();

        ctx.set_ini_filename(None);

        {
            let io = ctx.io_mut();
            io.display_size = size.into();
            io.display_framebuffer_scale = [1.0, 1.0];
        }

        self.contexts.insert(id, Some(ctx.suspend()));
    }

    pub fn unregister_window(&mut self, id: WindowId) {
        self.contexts.remove(&id);
    }

    pub fn is_registered(&self, id: WindowId) -> bool {
        self.contexts.contains_key(&id)
    }

    fn checkout(&mut self, id: WindowId) -> Result<imgui::Context, ImguiError> {
        let slot = self
            .contexts
            .get_mut(&id)
            .ok_or(ImguiError::NotRegistered(id))?;

        let suspended = slot.take().ok_or(ImguiError::NotRegistered(id))?;

        match suspended.activate() {
            Ok(ctx) => Ok(ctx),
            Err(s) => {
                *slot = Some(s);
                Err(ImguiError::ForeignContextActive)
            }
        }
    }

    fn checkin(&mut self, id: WindowId, ctx: imgui::Context) {
        if let Some(slot) = self.contexts.get_mut(&id) {
            *slot = Some(ctx.suspend());
        }
    }
}

pub struct ActiveImgui<'a> {
    lock: MutexGuard<'a, ImguiManager>,
    id: WindowId,
    ctx: Option<imgui::Context>,
}

impl ActiveImgui<'_> {
    pub fn window_id(&self) -> WindowId {
        self.id
    }
}

impl Deref for ActiveImgui<'_> {
    type Target = imgui::Context;

    fn deref(&self) -> &Self::Target {
        self.ctx.as_ref().expect("Context taken before drop")
    }
}

impl DerefMut for ActiveImgui<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx.as_mut().expect("Context taken before drop")
    }
}

impl Drop for ActiveImgui<'_> {
    fn drop(&mut self) {
        if let Some(ctx) = self.ctx.take() {
            self.lock.checkin(self.id, ctx);
        }
    }
}

#[derive(Default)]
#[derive(Clone)]
pub struct SharedImgui {
    manager: Arc<Mutex<ImguiManager>>,
}

unsafe impl Send for SharedImgui {}

impl SharedImgui {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_window(&self, id: WindowId, size: math::Size<f32>) {
        self.manager.lock().register_window(id, size);
    }

    pub fn unregister_window(&self, id: WindowId) {
        self.manager.lock().unregister_window(id);
    }

    pub fn is_registered(&self, id: WindowId) -> bool {
        self.manager.lock().is_registered(id)
    }

    pub fn try_active(&self, id: WindowId) -> Result<ActiveImgui<'_>, ImguiError> {
        let mut lock = self.manager.lock();
        let ctx = lock.checkout(id)?;

        Ok(ActiveImgui {
            lock,
            id,
            ctx: Some(ctx),
        })
    }

    pub fn active(&self, id: WindowId) -> ActiveImgui<'_> {
        self.try_active(id)
            .expect("Failed to activate imgui context")
    }

    pub fn with_active<R>(&self, id: WindowId, f: impl FnOnce(&mut imgui::Context) -> R) -> R {
        f(&mut self.active(id))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImguiCmd {
    pub clip: [f32; 4],
    pub vtx_offset: i32,
    pub idx_offset: u32,
    pub count: u32,
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct ImguiPacket {
    pub vertices: Vec<Vertex>,
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

        let (ox, oy, ew, eh) = (font_uv.x, font_uv.y, font_uv.z, font_uv.w);

        for list in dd.draw_lists() {
            let vtx_base = self.vertices.len() as i32;
            let idx_base = self.indices.len() as u32;

            self.vertices.extend(list.vtx_buffer().iter().map(|v| {
                let uv = math::Vector2::new(ox + v.uv[0] * ew, oy + v.uv[1] * eh);
                let col = math::Vector4::new(
                    v.col[0] as f32 / 255.0,
                    v.col[1] as f32 / 255.0,
                    v.col[2] as f32 / 255.0,
                    v.col[3] as f32 / 255.0,
                );

                Vertex::new(math::Vector3::new(v.pos[0], v.pos[1], 0.0), col, uv)
            }));

            self.indices
                .extend(list.idx_buffer().iter().map(|&i| i as u32)); // Uint32, like your Batcher

            for cmd in list.commands() {
                if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
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

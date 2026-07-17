pub mod input;

use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;

use ::logging::debug;
use ::logging::warn;
pub use dear_imgui_rs as imgui;
pub use imgui::*;
use parking_lot::Mutex;
use parking_lot::MutexGuard;
use utils::FastHashMap;
use winit::window::WindowId;

pub use crate::input::winit_keycode_to_imgui;
pub use crate::input::winit_mousebutton_to_imgui;

struct Clipboard(arboard::Clipboard);

impl imgui::ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<String> {
        self.0.get_text().ok()
    }

    fn set(&mut self, text: &str) {
        if let Err(e) = self.0.set_text(text) {
            warn!("clipboard set failed: {e}");
            return;
        }

        debug!("Text copied to clipboard.");
    }
}

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

        #[cfg(not(target_arch = "wasm32"))]
        match arboard::Clipboard::new() {
            Ok(cb) => ctx.set_clipboard_backend(Clipboard(cb)),
            Err(e) => warn!("clipboard unavailable: {e}"),
        }

        let _ = ctx.set_ini_filename::<PathBuf>(None);

        {
            let io = ctx.io_mut();
            io.set_display_size(size.into());
            io.set_display_framebuffer_scale([1.0, 1.0]);
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

/// Builds the font atlas with every glyph range pre-baked (the pre-1.92
/// full-build behavior) and returns its RGBA32 pixels and size.
///
/// The engine uploads the atlas once into its own texture atlas and remaps
/// UVs, so it can't service the incremental texture updates of the newer
/// backend protocol; pre-baking keeps the one-shot upload complete.
pub fn bake_font_atlas(ctx: &mut imgui::Context) -> (Vec<u8>, math::Size<u32>) {
    let mut fonts = ctx.font_atlas_mut();

    unsafe {
        let raw = fonts.raw();
        (*raw).TexDesiredFormat = sys::ImTextureFormat_RGBA32;
        sys::igImFontAtlasBuildMain(raw);
        sys::igImFontAtlasBuildLegacyPreloadAllGlyphRanges(raw);
    }

    let tex = fonts
        .tex_data_mut()
        .expect("font atlas build produced no texture");

    debug_assert_eq!(tex.format(), texture::TextureFormat::RGBA32);

    let pixels = tex
        .pixels()
        .expect("font atlas texture has no pixel data")
        .to_vec();

    (pixels, math::Size::new(tex.width(), tex.height()))
}

#[derive(Debug, Clone, Copy)]
pub struct ImguiCmd {
    pub clip: [f32; 4],
    pub vtx_offset: i32,
    pub idx_offset: u32,
    pub count: u32,
}

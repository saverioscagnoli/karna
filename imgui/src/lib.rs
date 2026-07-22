mod input;

use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;

use ::logging::debug;
use ::logging::warn;
use parking_lot::Mutex;
use parking_lot::MutexGuard;
use sdl3::clipboard::ClipboardUtil;
use utils::FastHashMap;

pub use dear_imgui_rs as imgui;
pub use imgui::*;

pub use crate::input::sdl_mousebutton_to_imgui;
pub use crate::input::sdl_scancode_to_imgui;
pub use crate::input::sdl_scancode_to_imgui_mod;

struct Clipboard(Arc<ClipboardUtil>);

impl imgui::ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<String> {
        self.0.clipboard_text().ok()
    }

    fn set(&mut self, value: &str) {
        if let Err(e) = self.0.set_clipboard_text(value) {
            warn!("Failed to copy to clipboard: {}", e);
            return;
        }

        debug!("Text copied to clipboard.");
    }
}

pub struct ImguiManager {
    contexts: FastHashMap<u32, Option<imgui::SuspendedContext>>,
}

impl ImguiManager {
    pub fn new() -> Self {
        Self {
            contexts: FastHashMap::default(),
        }
    }

    pub fn register_window(
        &mut self,
        id: u32,
        size: math::Size<u32>,
        sdl_clipboard: Arc<ClipboardUtil>,
    ) {
        let mut context = imgui::Context::create();

        context.set_clipboard_backend(Clipboard(sdl_clipboard));

        let _ = context.set_ini_filename::<PathBuf>(None);
        let io = context.io_mut();

        io.set_display_size(size.as_f32().into());
        io.set_display_framebuffer_scale([1.0, 1.0]);

        // The engine renderer services ImTextureData requests (imgui 1.92
        // texture protocol), so imgui must route textures through it.
        io.set_backend_flags(io.backend_flags() | imgui::BackendFlags::RENDERER_HAS_TEXTURES);

        self.contexts.insert(id, Some(context.suspend()));
        debug!("Registered imgui context for window '{}'", id);
    }

    pub fn unregister_window(&mut self, id: u32) {
        self.contexts.remove(&id);
        debug!("Unregistered imgui context for window '{}'", id);
    }

    pub fn is_registered(&self, id: u32) -> bool {
        self.contexts.contains_key(&id)
    }

    fn checkout(&mut self, id: u32) -> Result<imgui::Context, String> {
        let slot = self.contexts.get_mut(&id).ok_or(format!(
            "No imgui context is registered for window '{}'",
            id
        ))?;

        let suspended = slot.take().ok_or(format!(
            "No imgui context is registered for window '{}'",
            id
        ))?;

        match suspended.activate() {
            Ok(ctx) => Ok(ctx),
            Err(s) => {
                *slot = Some(s);
                Err(String::from("Foreign context active"))
            }
        }
    }

    fn checkin(&mut self, id: u32, context: imgui::Context) {
        if let Some(slot) = self.contexts.get_mut(&id) {
            *slot = Some(context.suspend())
        }
    }
}

pub struct ActiveImgui<'a> {
    lock: MutexGuard<'a, ImguiManager>,
    id: u32,
    context: Option<imgui::Context>,
}

impl Deref for ActiveImgui<'_> {
    type Target = imgui::Context;

    fn deref(&self) -> &Self::Target {
        self.context.as_ref().expect("Context taken before drop")
    }
}

impl DerefMut for ActiveImgui<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context.as_mut().expect("Context taken before drop")
    }
}

impl Drop for ActiveImgui<'_> {
    fn drop(&mut self) {
        if let Some(context) = self.context.take() {
            self.lock.checkin(self.id, context);
        }
    }
}

#[derive(Clone)]
pub struct SharedImgui {
    manager: Arc<Mutex<ImguiManager>>,
}

unsafe impl Send for SharedImgui {}

impl SharedImgui {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(ImguiManager::new())),
        }
    }

    pub fn register_window(
        &self,
        id: u32,
        size: math::Size<u32>,
        sdl_clipboard: Arc<ClipboardUtil>,
    ) {
        self.manager.lock().register_window(id, size, sdl_clipboard);
    }

    pub fn unregister_window(&self, id: u32) {
        self.manager.lock().unregister_window(id);
    }

    pub fn is_registered(&self, id: u32) -> bool {
        self.manager.lock().is_registered(id)
    }

    pub fn try_active(&self, id: u32) -> Result<ActiveImgui<'_>, String> {
        let mut lock = self.manager.lock();
        let ctx = lock.checkout(id)?;

        Ok(ActiveImgui {
            lock,
            id,
            context: Some(ctx),
        })
    }

    pub fn active(&self, id: u32) -> ActiveImgui<'_> {
        self.try_active(id)
            .expect("Failed to activate imgui context")
    }
}

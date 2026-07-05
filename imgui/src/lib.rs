use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

// Re-export imgui
pub use imgui::*;
use parking_lot::Mutex;
use parking_lot::MutexGuard;
use utils::FastHashMap;
use winit::window::WindowId;

enum Slot {
    Active(imgui::Context),
    Suspended(imgui::SuspendedContext),
}

pub struct ImguiManager {
    slots: FastHashMap<WindowId, Slot>,
    active_window: Option<WindowId>,
}

impl ImguiManager {
    pub fn new() -> Self {
        Self {
            slots: FastHashMap::default(),
            active_window: None,
        }
    }

    fn suspend_active(&mut self) {
        if let Some(id) = self.active_window.take()
            && let Some(Slot::Active(ctx)) = self.slots.remove(&id)
        {
            self.slots.insert(id, Slot::Suspended(ctx.suspend()));
        }
    }

    pub fn register_window(&mut self, id: WindowId) {
        // Suspend the active context, if any,
        // because you can't have more than one active
        // imgui context at a time
        self.suspend_active();

        let mut ctx = imgui::Context::create();

        ctx.set_ini_filename(None);

        self.slots.insert(id, Slot::Suspended(ctx.suspend()));
    }

    pub fn unregister_window(&mut self, id: WindowId) {
        if self.active_window == Some(id) {
            self.active_window = None;
        }

        self.slots.remove(&id);
    }

    fn activate(&mut self, id: WindowId) {
        if self.active_window == Some(id) {
            return;
        }

        self.suspend_active();
        let slot = self.slots.remove(&id).expect("Window not registered");

        match slot {
            Slot::Suspended(s) => match s.activate() {
                Ok(ctx) => {
                    self.slots.insert(id, Slot::Active(ctx));
                    self.active_window = Some(id);
                }
                Err(s) => {
                    self.slots.insert(id, Slot::Suspended(s));
                    panic!("Imgui activate failed for {id:?}");
                }
            },
            Slot::Active(ctx) => {
                self.slots.insert(id, Slot::Active(ctx));
                self.active_window = Some(id);
            }
        }
    }

    fn get(&self, id: WindowId) -> &imgui::Context {
        match self.slots.get(&id).unwrap() {
            Slot::Active(c) => c,
            _ => unreachable!(),
        }
    }

    fn get_mut(&mut self, id: WindowId) -> &mut imgui::Context {
        match self.slots.get_mut(&id).unwrap() {
            Slot::Active(c) => c,
            _ => unreachable!(),
        }
    }
}

/// `activate` guarantees that only one imgui context is ever the current one
unsafe impl Send for ImguiManager {}

#[derive(Clone)]
pub struct SharedImgui {
    inner: Arc<Mutex<ImguiManager>>,
}

impl SharedImgui {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ImguiManager::new())),
        }
    }

    pub fn guard<'a>(&'a self) -> MutexGuard<'a, ImguiManager> {
        self.inner.lock()
    }
}

pub struct ActiveImgui<'a> {
    guard: MutexGuard<'a, ImguiManager>,
    id: WindowId,
}

impl<'a> ActiveImgui<'a> {
    pub fn new(shared: &'a SharedImgui, id: WindowId) -> Self {
        let mut guard = shared.guard();
        guard.activate(id);
        Self { guard, id }
    }
}

impl<'a> Deref for ActiveImgui<'a> {
    type Target = imgui::Context;

    fn deref(&self) -> &Self::Target {
        self.guard.get(self.id)
    }
}

impl<'a> DerefMut for ActiveImgui<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.get_mut(self.id)
    }
}

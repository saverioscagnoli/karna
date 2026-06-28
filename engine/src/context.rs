use crate::window::Window;

pub struct WindowContext {
    pub window: Window,
}

unsafe impl Send for WindowContext {}
unsafe impl Sync for WindowContext {}

pub struct ContextRefMut<'ctx> {
    pub window: &'ctx Window,
}

pub struct ContextRef<'ctx> {
    pub window: &'ctx Window,
}

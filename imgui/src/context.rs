use std::ptr;

use dear_imgui_sys::*;
use logging::debug;

pub struct ImguiContext {
    ptr: *mut ImGuiContext,
}

impl ImguiContext {
    pub fn new() -> Self {
        let raw = unsafe { igCreateContext(ptr::null_mut()) };
        assert!(!raw.is_null(), "Failed to create imgui context");

        unsafe {
            igSetCurrentContext(raw);

            let io = &mut *igGetIO_Nil();

            io.BackendFlags |= ImGuiBackendFlags_RendererHasTextures as i32;
            io.BackendFlags |= ImGuiBackendFlags_RendererHasVtxOffset as i32;
            io.IniFilename = ptr::null();
        }

        debug!("Imgui context created.");
        Self { ptr: raw }
    }

    pub fn set_current(&self) {
        unsafe { igSetCurrentContext(self.ptr) }
    }

    pub fn io(&self) -> *mut ImGuiIO {
        unsafe { igGetIO_Nil() }
    }
}

impl Drop for ImguiContext {
    fn drop(&mut self) {
        unsafe { igDestroyContext(self.ptr) };
        debug!("Imgui context destroyed.");
    }
}

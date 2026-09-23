use core::ptr::NonNull;

use quickjs_sys::JS_FreeRuntime;
use quickjs_sys::JS_NewRuntime;
use quickjs_sys::JS_RunGC;
use quickjs_sys::JS_SetMaxStackSize;
use quickjs_sys::JS_SetMemoryLimit;
use quickjs_sys::JSRuntime;

use crate::Error;

pub struct Runtime {
    raw: NonNull<JSRuntime>,
}

impl Runtime {
    pub fn new() -> Result<Self, Error> {
        let raw = NonNull::new(unsafe { JS_NewRuntime() }).ok_or(Error::OutOfMemory)?;

        Ok(Self { raw })
    }

    pub fn set_memory_limit(&self, bytes: usize) {
        unsafe { JS_SetMemoryLimit(self.as_ptr(), bytes) };
    }

    pub fn set_max_stack_size(&self, bytes: usize) {
        unsafe { JS_SetMaxStackSize(self.as_ptr(), bytes) };
    }

    pub fn run_gc(&self) {
        unsafe { JS_RunGC(self.as_ptr()) };
    }

    pub fn as_ptr(&self) -> *mut JSRuntime {
        self.raw.as_ptr()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { JS_FreeRuntime(self.as_ptr()) };
    }
}

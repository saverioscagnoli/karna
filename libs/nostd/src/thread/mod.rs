use core::cell::UnsafeCell;
use core::ffi;
use core::ffi::CStr;
use core::mem::ManuallyDrop;
use core::ptr;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use sdl3_sys::SDL_CreateThreadRuntime;
use sdl3_sys::SDL_DetachThread;
use sdl3_sys::SDL_Thread;
use sdl3_sys::SDL_WaitThread;
use sdl3_sys::get_error;

struct Packet<T>(UnsafeCell<Option<T>>);

unsafe impl<T: Send> Send for Packet<T> {}
unsafe impl<T: Send> Sync for Packet<T> {}

pub struct JoinHandle<T> {
    thread: ptr::NonNull<SDL_Thread>,
    packet: Arc<Packet<T>>,
}

unsafe extern "C" fn trampoline(data: *mut ffi::c_void) -> ffi::c_int {
    let f: Box<Box<dyn FnOnce() + Send>> = unsafe { Box::from_raw(data.cast()) };
    f();
    0
}

pub fn spawn<F, T>(name: &CStr, f: F) -> Result<JoinHandle<T>, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let packet = Arc::new(Packet(UnsafeCell::new(None)));
    let theirs = Arc::clone(&packet);

    let body: Box<dyn FnOnce() + Send> = Box::new(move || {
        let value = f();
        unsafe { *theirs.0.get() = Some(value) };
    });

    let data = Box::into_raw(Box::new(body));
    let thread = unsafe {
        SDL_CreateThreadRuntime(Some(trampoline), name.as_ptr(), data.cast(), None, None)
    };

    match ptr::NonNull::new(thread) {
        Some(thread) => Ok(JoinHandle { thread, packet }),
        None => {
            drop(unsafe { Box::from_raw(data) });
            Err(get_error().to_string())
        }
    }
}

unsafe impl<T: Send> Send for JoinHandle<T> {}
unsafe impl<T: Send> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Option<T> {
        let this = ManuallyDrop::new(self);
        unsafe { SDL_WaitThread(this.thread.as_ptr(), ptr::null_mut()) };

        // SAFETY: `this` is never dropped, so this is the only read of the field.
        let packet = unsafe { ptr::read(&this.packet) };
        unsafe { (*packet.0.get()).take() }
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        unsafe { SDL_DetachThread(self.thread.as_ptr()) };
    }
}

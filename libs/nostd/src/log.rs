use core::error;
use core::ffi::CStr;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::c_void;
use core::mem;
use core::ptr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::Acquire;
use core::sync::atomic::Ordering::Release;

use alloc::boxed::Box;

use sdl3_sys::SDL_GetDefaultLogOutputFunction;
use sdl3_sys::SDL_LOG_CATEGORY_APPLICATION;
use sdl3_sys::SDL_LOG_PRIORITY_CRITICAL;
use sdl3_sys::SDL_LOG_PRIORITY_DEBUG;
use sdl3_sys::SDL_LOG_PRIORITY_ERROR;
use sdl3_sys::SDL_LOG_PRIORITY_INFO;
use sdl3_sys::SDL_LOG_PRIORITY_TRACE;
use sdl3_sys::SDL_LOG_PRIORITY_VERBOSE;
use sdl3_sys::SDL_LOG_PRIORITY_WARN;
use sdl3_sys::SDL_LogCategory;
use sdl3_sys::SDL_LogPriority;
use sdl3_sys::SDL_ResetLogPriorities;
use sdl3_sys::SDL_SetLogOutputFunction;
use sdl3_sys::SDL_SetLogPriorities;
use sdl3_sys::SDL_SetLogPriorityPrefix;
use sdl3_sys::SdlError;

pub const SDL_TARGET: &str = "sdl";

const BUF: usize = 1024;

const PRIORITIES: [SDL_LogPriority; 7] = [
    SDL_LOG_PRIORITY_TRACE,
    SDL_LOG_PRIORITY_VERBOSE,
    SDL_LOG_PRIORITY_DEBUG,
    SDL_LOG_PRIORITY_INFO,
    SDL_LOG_PRIORITY_WARN,
    SDL_LOG_PRIORITY_ERROR,
    SDL_LOG_PRIORITY_CRITICAL,
];

type OutputFn = unsafe extern "C" fn(*mut c_void, c_int, SDL_LogPriority, *const c_char);

static DEFAULT_OUTPUT: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

pub struct SdlTarget {
    category: c_int,
}

impl SdlTarget {
    pub const fn new(category: SDL_LogCategory) -> Self {
        Self {
            category: category as c_int,
        }
    }
}

impl Default for SdlTarget {
    fn default() -> Self {
        Self::new(SDL_LOG_CATEGORY_APPLICATION)
    }
}

impl traccia::Target for SdlTarget {
    fn write(&self, level: traccia::Level, message: &str) -> Result<(), Box<dyn error::Error>> {
        let Some(output) = default_output() else {
            return Err(Box::new(SdlError::new(
                "SDL has no default log output function.",
            )));
        };

        let mut buf = [0u8; BUF];

        copy(message, &mut buf);

        unsafe {
            output(
                ptr::null_mut(),
                self.category,
                priority(level),
                buf.as_ptr().cast(),
            )
        };

        Ok(())
    }
}

pub fn capture() {
    unsafe {
        for priority in PRIORITIES {
            SDL_SetLogPriorityPrefix(priority, ptr::null());
        }

        SDL_SetLogPriorities(SDL_LOG_PRIORITY_TRACE);
        SDL_SetLogOutputFunction(Some(forward), ptr::null_mut());
    }
}

pub fn release() {
    unsafe {
        SDL_SetLogOutputFunction(SDL_GetDefaultLogOutputFunction(), ptr::null_mut());
        SDL_ResetLogPriorities();

        for priority in PRIORITIES {
            SDL_SetLogPriorityPrefix(priority, ptr::null());
        }
    }
}

unsafe extern "C" fn forward(
    _userdata: *mut c_void,
    _category: c_int,
    priority: SDL_LogPriority,
    message: *const c_char,
) {
    if message.is_null() {
        return;
    }

    let Ok(message) = (unsafe { CStr::from_ptr(message) }).to_str() else {
        return;
    };

    traccia::log!(target: SDL_TARGET, level(priority), "{}", message);
}

fn default_output() -> Option<OutputFn> {
    let stored = DEFAULT_OUTPUT.load(Acquire);

    if !stored.is_null() {
        return Some(unsafe { mem::transmute::<*mut (), OutputFn>(stored) });
    }

    let default = unsafe { SDL_GetDefaultLogOutputFunction() }?;

    DEFAULT_OUTPUT.store(default as *mut (), Release);

    Some(default)
}

fn copy(message: &str, buf: &mut [u8; BUF]) {
    let bytes = message.as_bytes();
    let mut n = bytes.len().min(BUF - 1);

    if n < bytes.len() {
        while n > 0 && bytes[n] & 0xc0 == 0x80 {
            n -= 1;
        }
    }

    for i in 0..n {
        buf[i] = if bytes[i] == 0 { b'?' } else { bytes[i] };
    }

    buf[n] = 0;
}

fn priority(level: traccia::Level) -> SDL_LogPriority {
    match level {
        traccia::Level::Trace => SDL_LOG_PRIORITY_TRACE,
        traccia::Level::Debug => SDL_LOG_PRIORITY_DEBUG,
        traccia::Level::Info => SDL_LOG_PRIORITY_INFO,
        traccia::Level::Warn => SDL_LOG_PRIORITY_WARN,
        traccia::Level::Error => SDL_LOG_PRIORITY_ERROR,
    }
}

fn level(priority: SDL_LogPriority) -> traccia::Level {
    match priority {
        SDL_LOG_PRIORITY_TRACE | SDL_LOG_PRIORITY_VERBOSE => traccia::Level::Trace,
        SDL_LOG_PRIORITY_DEBUG => traccia::Level::Debug,
        SDL_LOG_PRIORITY_WARN => traccia::Level::Warn,
        SDL_LOG_PRIORITY_ERROR | SDL_LOG_PRIORITY_CRITICAL => traccia::Level::Error,
        _ => traccia::Level::Info,
    }
}

#![no_std]

extern crate alloc;

pub mod gpu;
pub mod math;
pub mod window;

use core::marker::PhantomData;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::AcqRel;
use core::sync::atomic::Ordering::Release;

use alloc::format;
use sdl3_sys::SDL_GetVersion;
use sdl3_sys::SDL_INIT_VIDEO;
use sdl3_sys::SDL_Init;
use sdl3_sys::SDL_MAJOR_VERSION;
use sdl3_sys::SDL_MICRO_VERSION;
use sdl3_sys::SDL_MINOR_VERSION;
use sdl3_sys::SDL_Quit;
use sdl3_sys::SdlError;

pub use sdl3_sys::get_error;

pub use crate::alloc::string::String;

static SDL_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct SdlGuard(PhantomData<*const ()>);

impl SdlGuard {
    pub fn init() -> Result<Self, SdlError> {
        if SDL_ACTIVE.swap(true, AcqRel) {
            return Err(SdlError::new("SDL was initialized more tha one time."));
        }

        unsafe {
            if !SDL_Init(SDL_INIT_VIDEO) {
                return Err(get_error());
            }
        }

        Ok(Self(PhantomData))
    }
}

impl Drop for SdlGuard {
    fn drop(&mut self) {
        unsafe { SDL_Quit() };

        SDL_ACTIVE.store(false, Release);
    }
}

pub const COMPILED_VERSION: i32 = versionnum(
    SDL_MAJOR_VERSION as i32,
    SDL_MINOR_VERSION as i32,
    SDL_MICRO_VERSION as i32,
);

pub const fn versionnum(major: i32, minor: i32, patch: i32) -> i32 {
    major * 1_000_000 + minor * 1_000 + patch
}

pub const fn version_major(v: i32) -> i32 {
    v / 1_000_000
}
pub const fn version_minor(v: i32) -> i32 {
    (v / 1_000) % 1_000
}

pub const fn version_micro(v: i32) -> i32 {
    v % 1_000
}

pub const fn version_atleast(major: i32, minor: i32, patch: i32) -> bool {
    COMPILED_VERSION >= versionnum(major, minor, patch)
}

pub fn linked_version() -> String {
    let v = unsafe { SDL_GetVersion() };
    format!(
        "{}.{}.{}",
        version_major(v),
        version_minor(v),
        version_micro(v)
    )
}

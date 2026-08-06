mod err;

use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use logging::debug;
use sdl3::SDL_GetVersion;
use sdl3::SDL_INIT_VIDEO;
use sdl3::SDL_Init;
use sdl3::SDL_Quit;

use crate::err::SDL_last_error;

static SDL_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct App {
    /// Makes app !Send and !Sync, pinning SDL
    /// to the thread that created it
    _sdl: PhantomData<*const ()>,
}

impl App {
    pub fn new() -> Result<Self, String> {
        if SDL_ACTIVE.swap(true, Ordering::AcqRel) {
            return Err("SDL is already initialized.".into());
        }

        let success = unsafe { SDL_Init(SDL_INIT_VIDEO) };

        if !success {
            SDL_ACTIVE.store(false, Ordering::Release);
            return Err(SDL_last_error());
        }

        debug!("SDL v{} initialized.", sdl3::compiled_version_string());

        Ok(Self { _sdl: PhantomData })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe { SDL_Quit() };
        SDL_ACTIVE.store(false, Ordering::Release);
    }
}

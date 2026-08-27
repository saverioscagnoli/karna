use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::AcqRel;

use sdl3::SDL_Quit;

pub static SDL_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn init_check() -> bool {
    SDL_ACTIVE.swap(true, AcqRel)
}

pub struct SDLGuard(PhantomData<*const ()>);

impl SDLGuard {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl Drop for SDLGuard {
    fn drop(&mut self) {
        unsafe { SDL_Quit() };
        SDL_ACTIVE.store(false, Ordering::Release);
    }
}

#![no_std]

mod err;

use core::cell::Cell;
use core::marker::PhantomData;
use core::ptr;

use engine::assets::AssetServer;
use engine::audio::AudioHandle;
use engine::input::Input;
use engine::render::Draw;
use engine::time::Time;
use engine::time::TimeData;
use engine::window::Window;
use engine::window::WindowData;

use crate::err::HostError;

/// In the engine often there's different
/// structs for immutable and mutable data,
/// for example the window, where the engine owns
/// the SDL window, while the user gets only a handle, where
/// they can perform requests.
/// So here a self-documenting pointer that differentiates
/// when the type is being used mutably or not is useful
enum Ptr<M, R> {
    Mut(ptr::NonNull<M>),
    Ref(ptr::NonNull<R>),
}

impl<M, R> Clone for Ptr<M, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M, R> Copy for Ptr<M, R> {}

impl<M, R> Ptr<M, R> {
    fn new<'a, N, S>(lent: Lent<'a, N, S>) -> Self {
        match lent {
            Lent::Mut(m) => Self::Mut(ptr::NonNull::from(m).cast()),
            Lent::Ref(r) => Self::Ref(ptr::NonNull::from(r).cast()),
        }
    }
}

enum Lent<'a, M, R = M> {
    Mut(&'a mut M),
    Ref(&'a R),
}

#[derive(Clone, Copy)]
pub struct Frame {
    window: Ptr<Window<'static>, WindowData>,
    time: Ptr<Time<'static>, TimeData>,
    input: ptr::NonNull<Input>,
    assets: Ptr<AssetServer, AssetServer>,
    audio: Option<ptr::NonNull<AudioHandle<'static>>>,
    draw: Option<ptr::NonNull<Draw<'static>>>,
}

impl Frame {
    fn new(
        window: Lent<'_, Window<'_>, WindowData>,
        time: Lent<'_, Time<'_>, TimeData>,
        input: &Input,
        assets: Lent<'_, AssetServer>,
        audio: Option<&mut AudioHandle<'_>>,
        draw: Option<&mut Draw<'_>>,
    ) -> Self {
        Self {
            window: Ptr::new(window),
            time: Ptr::new(time),
            input: ptr::NonNull::from(input),
            assets: Ptr::new(assets),
            audio: audio.map(|a| ptr::NonNull::from(a).cast()),
            draw: draw.map(|d| ptr::NonNull::from(d).cast()),
        }
    }
}

pub trait Lang {
    type Error;
    fn host_error(e: HostError) -> Self::Error;
}

pub struct Host<L: Lang> {
    frame: Cell<Option<Frame>>,
    _d: PhantomData<L>,
}

impl<L: Lang> Host<L> {
    fn frame(&self, what: &'static str) -> Result<Frame, L::Error> {
        self.frame
            .get()
            .ok_or_else(|| L::host_error(HostError::OutsideHook(what)))
    }

    pub fn window<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&WindowData) -> R,
    {
        // Window dereferences to window data anyway
        match self.frame("karna.window")?.window {
            Ptr::Ref(w) => Ok(f(unsafe { w.as_ref() })),
            Ptr::Mut(w) => Ok(f(unsafe { w.as_ref() })),
        }
    }

    pub fn window_mut<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&mut Window<'_>) -> R,
    {
        match self.frame("karna.window")?.window {
            Ptr::Ref(_) => Err(L::host_error(HostError::ReadOnly("karna.window"))),
            Ptr::Mut(mut w) => Ok(f(unsafe { w.as_mut() })),
        }
    }
}

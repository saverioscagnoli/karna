use core::ptr;

use engine::assets::AssetServer;
use engine::audio::AudioHandle;
use engine::input::Input;
use engine::render::Draw;
use engine::time::Time;
use engine::time::TimeData;
use engine::window::Window;
use engine::window::WindowData;

/// In the engine often there's different
/// structs for immutable and mutable data,
/// for example the window, where the engine owns
/// the SDL window, while the user gets only a handle, where
/// they can perform requests.
/// So here a self-documenting pointer that differentiates
/// when the type is being used mutably or not is useful
pub enum Ptr<M, R> {
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

pub enum Lent<'a, M, R = M> {
    Mut(&'a mut M),
    Ref(&'a R),
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub window: Ptr<Window<'static>, WindowData>,
    pub time: Ptr<Time<'static>, TimeData>,
    pub input: ptr::NonNull<Input>,
    pub assets: Ptr<AssetServer, AssetServer>,
    pub audio: Option<ptr::NonNull<AudioHandle<'static>>>,
    pub draw: Option<ptr::NonNull<Draw<'static>>>,
}

impl Frame {
    pub fn new(
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

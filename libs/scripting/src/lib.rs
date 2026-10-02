#![no_std]

mod err;
mod frame;
mod lang;
mod scene;

use core::cell::Cell;
use core::marker::PhantomData;

use engine::assets::AssetServer;
use engine::audio::AudioHandle;
use engine::context::DrawContext;
use engine::context::LoadContext;
use engine::context::UpdateContext;
use engine::input::Input;
use engine::render::Draw;
use engine::time::Time;
use engine::time::TimeData;
use engine::window::Window;
use engine::window::WindowData;

use crate::frame::Frame;
use crate::frame::Lent;
use crate::frame::Ptr;

pub use crate::err::HostError;
pub use crate::lang::Hook;
pub use crate::lang::Lang;
pub use crate::scene::ScriptScene;

pub struct Host<L: Lang> {
    frame: Cell<Option<Frame>>,
    _d: PhantomData<L>,
}

impl<L: Lang> Default for Host<L> {
    fn default() -> Self {
        Self {
            frame: Cell::new(None),
            _d: PhantomData,
        }
    }
}

impl<L: Lang> Host<L> {
    fn enter<R, F>(&self, frame: Frame, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        struct Restore<'a>(&'a Cell<Option<Frame>>, Option<Frame>);

        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                self.0.set(self.1);
            }
        }

        let _restore = Restore(&self.frame, self.frame.replace(Some(frame)));

        f()
    }

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

    pub fn time<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&TimeData) -> R,
    {
        match self.frame("karna.time")?.time {
            Ptr::Ref(t) => Ok(f(unsafe { t.as_ref() })),
            Ptr::Mut(t) => Ok(f(unsafe { t.as_ref() })),
        }
    }

    pub fn time_mut<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&mut Time<'_>) -> R,
    {
        match self.frame("karna.time")?.time {
            Ptr::Ref(_) => Err(L::host_error(HostError::ReadOnly("karna.time"))),
            Ptr::Mut(mut t) => Ok(f(unsafe { t.as_mut() })),
        }
    }

    pub fn input<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&Input) -> R,
    {
        Ok(f(unsafe { self.frame("karna.input")?.input.as_ref() }))
    }

    pub fn assets<F, R>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&AssetServer) -> R,
    {
        match self.frame("karna.assets")?.assets {
            Ptr::Ref(a) => Ok(f(unsafe { a.as_ref() })),
            Ptr::Mut(a) => Ok(f(unsafe { a.as_ref() })),
        }
    }

    pub fn assets_mut<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&mut AssetServer) -> R,
    {
        match self.frame("karna.assets")?.assets {
            Ptr::Ref(_) => Err(L::host_error(HostError::ReadOnly("karna.assets"))),
            Ptr::Mut(mut a) => Ok(f(unsafe { a.as_mut() })),
        }
    }

    pub fn audio_mut<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&mut AudioHandle<'_>) -> R,
    {
        let mut a = self
            .frame("karna.audio")?
            .audio
            .ok_or_else(|| L::host_error(HostError::Unavailable("karna.audio")))?;

        Ok(f(unsafe { a.as_mut() }))
    }

    pub fn draw<R, F>(&self, f: F) -> Result<R, L::Error>
    where
        F: FnOnce(&mut Draw<'_>) -> R,
    {
        let mut d = self
            .frame("draw handle")?
            .draw
            .ok_or_else(|| L::host_error(HostError::DrawOnly))?;

        Ok(f(unsafe { d.as_mut() }))
    }

    pub(crate) fn with_load<R, F>(&self, ctx: &mut LoadContext, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let frame = Frame::new(
            Lent::Mut(&mut ctx.window),
            Lent::Mut(&mut ctx.time),
            &ctx.input,
            Lent::Mut(&mut ctx.assets),
            Some(&mut ctx.audio),
            None,
        );

        self.enter(frame, f)
    }

    pub(crate) fn with_update<R, F>(&self, ctx: &mut UpdateContext, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let frame = Frame::new(
            Lent::Mut(&mut ctx.window),
            Lent::Mut(&mut ctx.time),
            &ctx.input,
            Lent::Mut(&mut ctx.assets),
            Some(&mut ctx.audio),
            None,
        );

        self.enter(frame, f)
    }

    pub(crate) fn with_draw<R, F>(&self, ctx: &DrawContext, draw: &mut Draw, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let frame = Frame::new(
            Lent::Ref(&ctx.window),
            Lent::Ref(&ctx.time),
            &ctx.input,
            Lent::Ref(&ctx.assets),
            None,
            Some(draw),
        );

        self.enter(frame, f)
    }
}

use core::cell::Cell;
use core::ptr::NonNull;

use engine::assets::AssetServer;
use engine::input::Input;
use engine::render::Draw;
use engine::time::Time;
use engine::time::TimeData;
use engine::window::Window;
use engine::window::WindowData;
use quickjs::Error;

/// What the engine lent the script for the hook currently running
#[derive(Default)]
pub(crate) struct Host {
    frame: Cell<Option<Frame>>,
}

#[derive(Clone, Copy)]
pub(crate) struct Frame {
    window: Ptr<Window<'static>, WindowData>,
    time: Ptr<Time<'static>, TimeData>,
    input: NonNull<Input>,
    assets: Ptr<AssetServer, AssetServer>,
    draw: Option<NonNull<Draw<'static>>>,
}

pub(crate) enum Lent<'a, M, R = M> {
    Mut(&'a mut M),
    Ref(&'a R),
}

enum Ptr<M, R> {
    Mut(NonNull<M>),
    Ref(NonNull<R>),
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
            Lent::Mut(m) => Self::Mut(NonNull::from(m).cast()),
            Lent::Ref(r) => Self::Ref(NonNull::from(r).cast()),
        }
    }
}

impl Frame {
    pub fn new(
        window: Lent<'_, Window<'_>, WindowData>,
        time: Lent<'_, Time<'_>, TimeData>,
        input: &Input,
        assets: Lent<'_, AssetServer>,
        draw: Option<&mut Draw<'_>>,
    ) -> Self {
        Self {
            window: Ptr::new(window),
            time: Ptr::new(time),
            input: NonNull::from(input),
            assets: Ptr::new(assets),
            draw: draw.map(|d| NonNull::from(d).cast()),
        }
    }
}

impl Host {
    pub fn enter<R>(&self, frame: Frame, f: impl FnOnce() -> R) -> R {
        struct Restore<'a>(&'a Cell<Option<Frame>>, Option<Frame>);

        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                self.0.set(self.1);
            }
        }

        let _restore = Restore(&self.frame, self.frame.replace(Some(frame)));

        f()
    }

    fn frame(&self, what: &str) -> Result<Frame, Error> {
        self.frame.get().ok_or_else(|| {
            Error::custom(nostd::alloc::format!(
                "{what} is only usable while the scene method it was passed to runs"
            ))
        })
    }

    pub fn window<R>(&self, f: impl FnOnce(&WindowData) -> R) -> Result<R, Error> {
        match self.frame("ctx.window")?.window {
            Ptr::Mut(w) => Ok(f(unsafe { w.as_ref() })),
            Ptr::Ref(w) => Ok(f(unsafe { w.as_ref() })),
        }
    }

    pub fn window_mut<R>(&self, f: impl FnOnce(&mut Window<'_>) -> R) -> Result<R, Error> {
        match self.frame("ctx.window")?.window {
            Ptr::Mut(mut w) => Ok(f(unsafe { w.as_mut() })),
            Ptr::Ref(_) => Err(Error::custom("the window cannot be changed during draw()")),
        }
    }

    pub fn time<R>(&self, f: impl FnOnce(&TimeData) -> R) -> Result<R, Error> {
        match self.frame("ctx.time")?.time {
            Ptr::Mut(t) => Ok(f(unsafe { t.as_ref() })),
            Ptr::Ref(t) => Ok(f(unsafe { t.as_ref() })),
        }
    }

    pub fn time_mut<R>(&self, f: impl FnOnce(&mut Time<'_>) -> R) -> Result<R, Error> {
        match self.frame("ctx.time")?.time {
            Ptr::Mut(mut t) => Ok(f(unsafe { t.as_mut() })),
            Ptr::Ref(_) => Err(Error::custom("time cannot be changed during draw()")),
        }
    }

    pub fn input<R>(&self, f: impl FnOnce(&Input) -> R) -> Result<R, Error> {
        let input = self.frame("ctx.input")?.input;
        Ok(f(unsafe { input.as_ref() }))
    }

    pub fn assets_mut<R>(&self, f: impl FnOnce(&mut AssetServer) -> R) -> Result<R, Error> {
        match self.frame("ctx.assets")?.assets {
            Ptr::Mut(mut a) => Ok(f(unsafe { a.as_mut() })),
            Ptr::Ref(_) => Err(Error::custom("assets cannot be loaded during draw()")),
        }
    }

    pub fn draw<R>(&self, f: impl FnOnce(&mut Draw<'_>) -> R) -> Result<R, Error> {
        let mut draw = self
            .frame("the graphics object")?
            .draw
            .ok_or_else(|| Error::custom("drawing is only possible inside draw()"))?;

        Ok(f(unsafe { draw.as_mut() }))
    }
}

use core::cell::Cell;
use core::ptr::NonNull;

use engine::assets::AssetServer;
use engine::input::Input;
use engine::render::Draw;
use engine::time::Time;
use engine::window::Window;
use quickjs::Error;

/// What the engine lent the script for the hook currently running
#[derive(Default)]
pub(crate) struct Host {
    frame: Cell<Option<Frame>>,
}

#[derive(Clone, Copy)]
pub(crate) struct Frame {
    window: NonNull<Window<'static>>,
    time: NonNull<Time<'static>>,
    input: NonNull<Input>,
    assets: NonNull<AssetServer>,
    assets_mut: bool,
    draw: Option<NonNull<Draw<'static>>>,
}

pub(crate) enum Assets<'a> {
    Mut(&'a mut AssetServer),
    Ref(&'a AssetServer),
}

impl Frame {
    pub fn new(
        window: &mut Window<'_>,
        time: &mut Time<'_>,
        input: &Input,
        assets: Assets<'_>,
        draw: Option<&mut Draw<'_>>,
    ) -> Self {
        let (assets, assets_mut) = match assets {
            Assets::Mut(a) => (NonNull::from(a), true),
            Assets::Ref(a) => (NonNull::from(a), false),
        };

        Self {
            window: NonNull::from(window).cast(),
            time: NonNull::from(time).cast(),
            input: NonNull::from(input),
            assets,
            assets_mut,
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

    pub fn window<R>(&self, f: impl FnOnce(&mut Window<'_>) -> R) -> Result<R, Error> {
        let mut window = self.frame("ctx.window")?.window;
        Ok(f(unsafe { window.as_mut() }))
    }

    pub fn time<R>(&self, f: impl FnOnce(&mut Time<'_>) -> R) -> Result<R, Error> {
        let mut time = self.frame("ctx.time")?.time;
        Ok(f(unsafe { time.as_mut() }))
    }

    pub fn input<R>(&self, f: impl FnOnce(&Input) -> R) -> Result<R, Error> {
        let input = self.frame("ctx.input")?.input;
        Ok(f(unsafe { input.as_ref() }))
    }

    pub fn assets_mut<R>(&self, f: impl FnOnce(&mut AssetServer) -> R) -> Result<R, Error> {
        let frame = self.frame("ctx.assets")?;

        if !frame.assets_mut {
            return Err(Error::custom("assets cannot be loaded during draw()"));
        }

        let mut assets = frame.assets;
        Ok(f(unsafe { assets.as_mut() }))
    }

    pub fn draw<R>(&self, f: impl FnOnce(&mut Draw<'_>) -> R) -> Result<R, Error> {
        let mut draw = self
            .frame("the graphics object")?
            .draw
            .ok_or_else(|| Error::custom("drawing is only possible inside draw()"))?;

        Ok(f(unsafe { draw.as_mut() }))
    }
}

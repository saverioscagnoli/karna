use crate::{
    context::Context,
    flags::LoopFlag,
    info,
    math::Size,
    traits::{Load, Render, ToU32, Update},
    warn,
};
use anyhow::anyhow;
use sdl2::event::Event;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

struct Sdl {
    _sdl: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    event_pump: sdl2::EventPump,
}

impl Sdl {
    fn new() -> anyhow::Result<Self> {
        let sdl = sdl2::init().map_err(|e| anyhow!(e))?;
        let video = sdl.video().map_err(|e| anyhow!(e))?;
        let event_pump = sdl.event_pump().map_err(|e| anyhow!(e))?;

        info!(
            "SDL version: {}, Karna version: {}",
            sdl2::version::version(),
            env!("CARGO_PKG_VERSION")
        );

        Ok(Self {
            _sdl: sdl,
            video,
            event_pump,
        })
    }
}

pub struct App {
    sdl: Sdl,
    ctx: Option<Context>,
    flags: HashSet<LoopFlag>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            sdl: Sdl::new()?,
            ctx: None,
            flags: HashSet::new(),
        })
    }

    pub fn flags(mut self, flags: &[LoopFlag]) -> Self {
        self.flags = flags.iter().cloned().collect();

        self
    }

    pub fn window<T: ToString, S: Into<Size>>(mut self, title: T, size: S) -> Self {
        let size = size.into();
        let Size { width, height } = size;

        let window = crate::window::Window::new(title.to_string(), width, height, &self.sdl.video);
        let ctx = Context::new(window, &self.flags);

        self.ctx = Some(ctx);

        self
    }

    pub fn run<S: Load + Update + Render>(&mut self, state: &mut S) {
        let ctx = self.ctx.as_mut().unwrap();

        ctx.running = true;

        state.load(ctx);

        // Show the window after loading, so the changes arent displayed in the making.
        ctx.window.show();

        let mut t0 = Instant::now();
        let mut acc = 0.0;

        let mut ticks = 0;
        let mut sec_timer = Instant::now();

        while ctx.running {
            let t1 = Instant::now();
            let dt = t1.duration_since(t0).as_secs_f32();

            t0 = t1;
            acc += dt;

            ctx.time.delta = dt;

            if sec_timer.elapsed().as_secs_f32() >= 1.0 {
                ctx.time.tps = ticks;
                ctx.time.fps = (1.0 / dt).round().to_u32();

                ticks = 0;
                sec_timer = Instant::now();
            }

            Self::handle_events(ctx, &mut self.sdl.event_pump);

            while acc >= ctx.time.tick_step {
                state.fixed_update(ctx);
                acc -= ctx.time.tick_step;

                ticks += 1;
            }

            state.update(ctx);

            ctx.render.clear();

            state.render(ctx);

            ctx.render.present();

            ctx.input.flush();

            if !self.flags.contains(&LoopFlag::VSync) {
                let elapsed = t0.elapsed().as_secs_f32();
                let remaining = ctx.time.render_step - elapsed;

                if remaining > 0.0 {
                    spin_sleep::sleep(Duration::from_secs_f32(remaining));
                }
            }
        }
    }

    fn handle_events(ctx: &mut Context, event_pump: &mut sdl2::EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    warn!("Received quit event. Closing.");
                    ctx.running = false;
                }

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    ctx.input.keys.insert(key);
                    ctx.input.keys_pressed.insert(key);
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    ctx.input.keys.remove(&key);
                }

                Event::MouseMotion { x, y, .. } => {
                    ctx.input.mouse_position.set(x, y);
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    ctx.input.mouse_buttons.insert(mouse_btn);
                    ctx.input.mouse_buttons_pressed.insert(mouse_btn);
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    ctx.input.mouse_buttons.remove(&mouse_btn);
                }

                _ => {}
            }
        }
    }
}

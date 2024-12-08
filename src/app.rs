use crate::{
    math::Size,
    traits::{Draw, Load, Update},
    Context,
};
use sdl2::event::Event;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct App {
    ctx: Context,
}

impl App {
    pub fn new<T: ToString, S: Into<Size>>(title: T, size: S) -> Result<Self, String> {
        let ctx = Context::new(title, size)?;

        Ok(Self { ctx })
    }

    pub fn run<S: Load + Update + Draw>(&mut self, state: &mut S) {
        self.ctx.start();

        state.load(&mut self.ctx);

        self.ctx.window.show();

        let mut event_pump = self.ctx.sdl.sys.event_pump().unwrap();

        let mut t0 = Instant::now();
        let mut acc = 0.0;

        let mut times = VecDeque::new();

        while self.ctx.running {
            let t1 = Instant::now();
            let dt = t1.duration_since(t0).as_secs_f32();
            t0 = t1;

            acc += dt;

            self.ctx.time.delta = dt;

            while times.len() > 0 && times[0] <= t1 - Duration::from_millis(1000) {
                times.pop_front();
            }

            times.push_back(t1);

            self.ctx.time.fps = times.len() as u32;

            self.handle_events(&mut event_pump);

            while acc >= self.ctx.time.tick_step {
                state.fixed_update(&mut self.ctx);
                acc -= self.ctx.time.tick_step;
            }

            state.update(&mut self.ctx);

            self.ctx.render.clear();
            state.draw(&mut self.ctx);
            self.ctx.render.present();

            self.ctx.input.flush();

            let t2 = t1.elapsed().as_secs_f32();
            let remaining = self.ctx.time.render_step - t2;

            if remaining > 0.0 {
                spin_sleep::sleep(Duration::from_secs_f32(remaining));
            }
        }
    }

    fn handle_events(&mut self, event_pump: &mut sdl2::EventPump) {
        let ctx = &mut self.ctx;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => ctx.stop(),

                // Keyboard events
                Event::KeyDown {
                    keycode: Some(key),
                    repeat,
                    ..
                } => {
                    if !ctx.input.key_down(key) {
                        ctx.input.keys.push(key);
                    }

                    if !repeat && !ctx.input.key_pressed(key) {
                        ctx.input.pressed_keys.push(key);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    ctx.input.keys.retain(|&k| k != key);
                }

                // Mouse events
                Event::MouseButtonDown { mouse_btn, .. } => {
                    if !ctx.input.mouse_down(mouse_btn) {
                        ctx.input.mouse_buttons.push(mouse_btn);
                    }

                    if !ctx.input.mouse_clicked(mouse_btn) {
                        ctx.input.clicked_mouse_buttons.push(mouse_btn);
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    ctx.input.mouse_buttons.retain(|&b| b != mouse_btn);
                }

                Event::MouseMotion { x, y, .. } => {
                    ctx.input.mouse_position.set(x, y);
                }

                _ => {}
            }
        }
    }
}

use crate::{
    input, perf,
    render::{self, Renderer},
    traits::{Load, Render, Update},
    window,
};
use sdl2::{event::Event, EventPump};
use std::time::{Duration, Instant};

pub struct EventLoop {
    sdl: sdl2::Sdl,
    running: bool,
    renderer: Renderer,
}

impl EventLoop {
    pub fn new(title: impl ToString, width: u32, height: u32) -> Self {
        input::init();
        render::init();

        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        let window = video
            .window(&title.to_string(), width, height)
            .position_centered()
            .build()
            .unwrap();

        let window_clone = window.clone();

        unsafe {
            _ = window::WINDOW.set(window);
        }

        let canvas = window_clone.into_canvas().accelerated().build().unwrap();
        let renderer = Renderer::new(canvas);

        Self {
            sdl,
            renderer,
            running: true,
        }
    }

    fn handle_events(&mut self, event_pump: &mut EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.running = false,
                Event::KeyDown {
                    keycode: Some(key),
                    repeat,
                    ..
                } => unsafe {
                    input::KEYS.get_mut().unwrap().insert(key);
                    input::KEYS_SINGLE_WITH_REPEAT
                        .get_mut()
                        .unwrap()
                        .insert(key);

                    if !repeat {
                        input::KEYS_SINGLE.get_mut().unwrap().insert(key);
                    }
                },

                Event::KeyUp {
                    keycode: Some(key), ..
                } => unsafe {
                    input::KEYS.get_mut().unwrap().remove(&key);
                },

                Event::MouseMotion { x, y, .. } => unsafe {
                    input::MOUSE_POSITION.get_mut().unwrap().set(x, y);
                },

                Event::MouseButtonDown { mouse_btn, .. } => unsafe {
                    input::MOUSE_BUTTONS.get_mut().unwrap().insert(mouse_btn);
                    input::MOUSE_BUTTONS_SINGLE
                        .get_mut()
                        .unwrap()
                        .insert(mouse_btn);
                },

                Event::MouseButtonUp { mouse_btn, .. } => unsafe {
                    input::MOUSE_BUTTONS.get_mut().unwrap().remove(&mouse_btn);
                },
                _ => {}
            }
        }
    }

    pub fn run<G>(&mut self, mut state: G)
    where
        G: Load + Update + Render,
    {
        let mut event_pump = self.sdl.event_pump().unwrap();

        let mut t0 = Instant::now();

        let mut acc = 0.0;
        let mut updates = 0;

        let mut perf_timer = Instant::now();

        state.load(&mut self.renderer);

        while self.running {
            let t1 = Instant::now();
            let dt = t1.duration_since(t0).as_secs_f32();

            acc += dt;
            t0 = t1;

            self.handle_events(&mut event_pump);

            while acc > perf::update_step() {
                // Fixed update with target ups. see `perf::set_target_ups`. Default is 60.
                state.update(perf::update_step());

                acc -= perf::update_step();
                updates += 1;
            }

            if perf_timer.elapsed().as_secs_f32() >= 1.0 {
                unsafe {
                    perf::FPS = (1.0 / dt).round() as u32;
                    perf::UPS = updates;
                };

                updates = 0;
                perf_timer = Instant::now();
            }

            self.renderer.clear();

            state.render(&mut self.renderer);

            self.renderer.present();

            let t2 = t1.elapsed().as_secs_f32();
            let remaining = perf::frame_step() - t2;

            if remaining > 0.0 {
                spin_sleep::sleep(Duration::from_secs_f32(remaining));
            }
        }
    }
}

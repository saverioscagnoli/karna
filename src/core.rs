use crate::{
    input::{self, keys, keys_pressed, keys_pressed_with_repeat, set_mouse_position},
    perf::{render_step, set_cpu, set_fps, set_mem, set_ups, update_step},
    render::{self, Renderer},
    throw,
    traits::{Load, Render, Update},
    window::{self, window},
    Error,
};
use atomic_float::AtomicF32;
use sdl2::{event::Event, EventPump, Sdl};
use std::thread;
use std::time::{Duration, Instant};
use std::{
    cell::OnceCell,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessesToUpdate, RefreshKind, System};

pub struct EventLoop {
    sdl: OnceCell<Sdl>,
    running: bool,
}

impl EventLoop {
    /// Creates a new event loop.
    /// The event loop is the main structure that drives the app logic.
    /// It is responsible for handling window events, updating the app state,
    /// rendering the app, etc.
    pub fn new() -> Self {
        Self {
            sdl: OnceCell::new(),
            running: false,
        }
    }

    pub fn create_window(
        &mut self,
        title: impl ToString,
        width: u32,
        height: u32,
    ) -> Result<(), Error> {
        let sdl = self.sdl.get_or_init(|| {
            sdl2::init()
                .map_err(|_| Error::Sdl("Failed to initialize SDL.".to_string()))
                .map_err(|e| {
                    throw!(e);
                })
                .unwrap()
        });

        let video_subsys = sdl
            .video()
            .map_err(|_| Error::Sdl("Failed to initialize SDL.".to_string()))?;

        let window = video_subsys
            .window(&title.to_string(), width, height)
            .position_centered()
            .build()
            .map_err(|_| Error::Window("Failed to create window.".to_string()))?;

        window::init(window);
        input::init();

        Ok(())
    }

    fn handle_events(&mut self, event_pump: &mut EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.running = false,
                Event::KeyDown {
                    keycode: Some(key),
                    repeat,
                    ..
                } => {
                    keys().insert(key);
                    keys_pressed_with_repeat().insert(key);

                    if !repeat {
                        keys_pressed().insert(key);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    keys().remove(&key);
                }

                Event::MouseMotion { x, y, .. } => {
                    set_mouse_position(x, y);
                }

                _ => {}
            }
        }
    }

    pub fn run<G>(&mut self, mut state: G)
    where
        G: Load + Update + Render,
    {
        let sdl = self.sdl.get();

        if sdl.is_none() {
            throw!(crate::Error::Sdl(
                "You need to create a window before running the event loop.".to_string()
            ));
        }

        let sdl = sdl.unwrap();

        self.running = true;

        let mut event_pump = sdl.event_pump().unwrap();
        let mut t0 = Instant::now();

        let mut acc = 0.0;

        let dt = Arc::new(AtomicF32::new(0.0));
        let ups = Arc::new(AtomicU32::new(0));

        let dt_clone = Arc::clone(&dt);
        let ups_clone = Arc::clone(&ups);

        thread::spawn(move || {
            let mut sys = System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                    .with_memory(MemoryRefreshKind::new().with_ram()),
            );

            let pid = sysinfo::get_current_pid().unwrap();
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]));

            loop {
                thread::sleep(Duration::from_secs(1));

                let fps = (1.0 / dt_clone.load(Ordering::Relaxed)).round();
                let ups = ups_clone.load(Ordering::Relaxed);

                // Refresh process
                sys.refresh_processes(ProcessesToUpdate::Some(&[pid]));
                let process = sys.process(pid).unwrap();

                let cpu = process.cpu_usage();
                let mem = process.memory();

                set_fps(fps);
                set_ups(ups);
                set_cpu(cpu);
                set_mem(mem as f32);

                ups_clone.store(0, Ordering::Relaxed);
            }
        });

        let canvas = window()
            .clone()
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();

        render::init(canvas.texture_creator());

        let mut renderer = Renderer::new(canvas);

        state.load(&mut renderer);

        while self.running {
            let t1 = Instant::now();
            let delta = t1.duration_since(t0).as_secs_f32();

            dt.store(delta, Ordering::Relaxed);

            acc += delta;
            t0 = t1;

            self.handle_events(&mut event_pump);

            let us = update_step();
            let rs = render_step();

            while acc > us {
                state.update(us);

                acc -= us;
                ups.fetch_add(1, Ordering::Relaxed);
            }

            renderer.clear();

            state.render(&mut renderer);

            renderer.present();

            let frame_time = t1.elapsed().as_secs_f32();
            let remaining = rs - frame_time;

            if remaining > 0.0 {
                spin_sleep::sleep(Duration::from_secs_f32(remaining));
            }
        }
    }
}

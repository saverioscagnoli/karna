mod logs;
mod render;
mod scene;
mod simulation;
mod window;

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};

use gpu::Gpu;
use logging::warn;
use logging::{error, info};
use utils::{FastHashMap, WindowId};

use crate::simulation::SimulationRunner;
use crate::window::{SdlEvent, SdlWindowEvent, WindowSlot};
use crate::window::{WindowAction, WindowHandle};

pub use crate::logs::init_logging;

struct Senders {
    shutdown: Sender<()>,
}

struct Receivers {
    action: Receiver<WindowAction>,
}

// Drop order matters
pub struct App {
    windows: FastHashMap<WindowId, WindowSlot>,
    sims: Option<JoinHandle<()>>,
    action_sender: Sender<WindowAction>,
    action_receiver: Receiver<WindowAction>,
    gpu: Gpu,
    events: sdl3::EventSubsystem,
    video: sdl3::VideoSubsystem,
    sdl: sdl3::Sdl,
}

impl App {
    pub fn new() -> Self {
        let sdl = sdl3::init().expect("Failed to init sdl3");
        let video = sdl.video().expect("Failed to init video subsystem");
        let events = sdl.event().expect("Failed to init event subsystem");
        let gpu = Gpu::init();

        let (action_tx, action_rx) = channel::<WindowAction>();

        gpu.log_shader_formats();
        gpu.log_info();

        Self {
            windows: FastHashMap::default(),
            sims: None,
            action_sender: action_tx,
            action_receiver: action_rx,
            gpu,
            events,
            video,
            sdl,
        }
    }

    fn spawn_sims(&mut self, specs: Vec<(String, math::Size<u32>)>) {
        let (event_tx, event_rx) = channel::<SdlEvent>();
        let action_tx = self.action_sender.clone();

        let mut handles = Vec::new();

        for (title, size) in specs {
            let window = self
                .video
                .window(&title, size.width, size.height)
                .build()
                .expect("Failed to create window");

            self.gpu
                .claim_window(&window)
                .expect("Failed to claim window");

            let id = window.id();

            handles.push((id, title.clone(), size));

            self.windows.insert(
                id,
                WindowSlot {
                    inner: window,
                    events: event_tx.clone(),
                },
            );
        }

        self.sims = Some(thread::spawn(move || {
            let mut runner = SimulationRunner::new(event_rx);

            for (id, title, size) in handles {
                runner.insert_window(WindowHandle {
                    action_sender: action_tx.clone(),
                    id,
                    cached_title: title.into(),
                    cached_size: size,
                });
            }

            runner.run();
        }));
    }

    fn close_window(&mut self, id: WindowId) {
        if let Some(slot) = self.windows.remove(&id) {
            info!("Closing window '{}'", id);
            self.gpu.release_window(&slot.inner);
        }
    }

    pub fn run(mut self) {
        self.spawn_sims(vec![(String::from("Demo"), math::Size::new(800, 600))]);
        let mut pump = self.sdl.event_pump().expect("Failed to get event pump");

        'main: loop {
            for event in pump.poll_iter() {
                match event {
                    SdlEvent::Quit { .. } => break 'main,
                    SdlEvent::Window {
                        timestamp,
                        window_id,
                        win_event,
                    } => {
                        let Some(slot) = self.windows.get(&window_id) else {
                            warn!("Received an event for a removed window");
                            continue;
                        };

                        let forwarded = SdlEvent::Window {
                            timestamp,
                            window_id,
                            win_event,
                        };

                        if let Err(e) = slot.events.send(forwarded) {
                            error!("Failed to send window event: {}", e);
                        }

                        if matches!(win_event, SdlWindowEvent::CloseRequested) {
                            self.close_window(window_id);

                            if self.windows.is_empty() {
                                break 'main;
                            }
                        }
                    }

                    SdlEvent::KeyDown { window_id, .. }
                    | SdlEvent::KeyUp { window_id, .. }
                    | SdlEvent::MouseMotion { window_id, .. }
                    | SdlEvent::MouseButtonDown { window_id, .. }
                    | SdlEvent::MouseButtonUp { window_id, .. }
                    | SdlEvent::MouseWheel { window_id, .. }
                    | SdlEvent::TextInput { window_id, .. } => {
                        if let Some(slot) = self.windows.get(&window_id) {
                            let _ = slot.events.send(event);
                        }
                    }

                    _ => {}
                }
            }

            while let Ok(action) = self.action_receiver.try_recv() {
                match action {
                    WindowAction::SetWindowTitle(id, title) => {
                        if let Some(slot) = self.windows.get_mut(&id) {
                            let _ = slot.inner.set_title(&title);
                        }
                    }
                    _ => {}
                }
            }

            for slot in self.windows.values() {
                draw(&self.gpu, &slot.inner);
            }
        }

        info!("All windows were closed. Exiting.");

        if let Some(t) = self.sims.take() {
            let _ = t.join();
        }
    }
}

use sdl3::gpu::{ColorTargetInfo, LoadOp, StoreOp};
use sdl3::pixels::Color;

fn draw(gpu: &Gpu, window: &sdl3::video::Window) {
    let mut cmd = match gpu.device.acquire_command_buffer() {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to acquire command buffer: {e}");
            return;
        }
    };

    // Can legitimately fail when the window is minimized — not an error.
    let Ok(swapchain) = cmd.wait_and_acquire_swapchain_texture(window) else {
        cmd.cancel();
        return;
    };

    let targets = [ColorTargetInfo::default()
        .with_texture(&swapchain)
        .with_load_op(LoadOp::CLEAR)
        .with_store_op(StoreOp::STORE)
        .with_clear_color(Color::RGB(30, 30, 46))];

    // An empty pass still clears — LoadOp::CLEAR runs regardless.
    let pass = gpu
        .device
        .begin_render_pass(&cmd, &targets, None)
        .expect("begin pass");
    gpu.device.end_render_pass(pass);

    cmd.submit().expect("Failed to submit");
}

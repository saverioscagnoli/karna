#![no_std]

pub mod builder;
pub mod context;
pub mod event;
pub mod render;
pub mod scene;
pub mod time;
pub mod window;
pub mod window_state;

use core::mem;

use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use nostd::collections::HashMap;
use nostd::log;
use nostd::mem::SdlAllocator;
use nostd::time::Instant;
use nostd::time::sleep_precise_until;
use sdl3::SdlGuard;
use sdl3::events::SdlEvent;
use sdl3::gpu::Device;
use sdl3::render::Color;
use sdl3::window::WindowId;
use traccia::info;
use traccia::trace;
use traccia::warn;

use crate::builder::WindowBuilder;
use crate::event::AppEvent;
use crate::event::AppOutboxes;
use crate::event::WindowEvent;
use crate::scene::SceneId;
use crate::time::Clock;
use crate::time::FramePacer;
use crate::time::PaceMode;
use crate::window::SdlWindow;
use crate::window_state::SceneSlot;
use crate::window_state::UpdatePhase;
use crate::window_state::WindowState;

#[global_allocator]
static ALLOC: SdlAllocator = SdlAllocator;

struct WindowEntry {
    state: WindowState,
    pacer: FramePacer,
    sdl_window: SdlWindow,
}

pub struct App {
    requested_windows: Vec<WindowBuilder>,
    windows: HashMap<WindowId, WindowEntry>,
    clock: Clock,
    device: Device,
    should_quit: bool,
    event_outboxes: AppOutboxes,
    event_queue: Vec<AppEvent>,
    _sdl: SdlGuard,
}

impl App {
    pub fn new(root: String) -> Self {
        let Ok(_sdl) = SdlGuard::init() else {
            panic!("sdl failed to init");
        };

        log::capture();
        info!("SDL v{} initialized.", sdl3::linked_version());

        let Ok(device) = sdl3::gpu::Device::init() else {
            panic!("failed to init gpu device");
        };

        let event_outboxes = AppOutboxes::new();

        Self {
            requested_windows: Vec::new(),
            windows: HashMap::default(),
            device,
            should_quit: false,
            clock: Clock::default(),
            event_queue: Vec::with_capacity(event_outboxes.total_cap()),
            event_outboxes,
            _sdl,
        }
    }

    fn spawn_window(&mut self, builder: WindowBuilder) {
        #[rustfmt::skip]
        let WindowBuilder { title, size, resizable, scene_builders, active_scenes } = builder;

        let sdl_window = self
            .device
            .create_window(title, size, resizable)
            .expect("Failed to create window");

        let scenes = scene_builders
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    SceneSlot {
                        builder: v,
                        scene: None,
                    },
                )
            })
            .collect::<HashMap<SceneId, SceneSlot>>();

        let state = WindowState::init(&sdl_window, scenes, active_scenes);

        self.windows.insert(
            sdl_window.id(),
            WindowEntry {
                state,
                pacer: FramePacer::new(PaceMode::Fixed),
                sdl_window,
            },
        );
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            return;
        };

        if self.windows.is_empty() {
            self.should_quit = true;
        }
    }

    fn quit(&mut self) {
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.close_window(id);
        }
    }

    fn drain_app_events(&mut self) {
        #[rustfmt::skip]
        let Self { windows, clock, event_outboxes, event_queue, .. } = self;

        event_outboxes.drain_into(event_queue);

        for event in event_queue.drain(..) {
            trace!("Received App event: {:?}", event);

            match event {
                AppEvent::SetTargetTPS(t) => clock.set_target_tps(t),
                AppEvent::Window { window, wevent } => {
                    let Some(entry) = windows.get_mut(&window) else {
                        warn!("Received app event for dropped window: {}", window);
                        continue;
                    };

                    match wevent {
                        WindowEvent::SetTitle(t) => entry.sdl_window.set_title(t),
                        WindowEvent::SetSize(s) => entry.sdl_window.set_size(s),
                        WindowEvent::SetTargetFPS(t) => entry.pacer.set_target_fps(t),
                        WindowEvent::SetFPSCalculationStrategy(s) => {
                            entry.pacer.counter.set_strategy(s)
                        }
                    }
                }
            }
        }
    }

    pub fn run(mut self) {
        for builder in mem::take(&mut self.requested_windows) {
            self.spawn_window(builder);
        }

        for entry in self.windows.values_mut() {
            entry.state.sync_time(&self.clock, &entry.pacer);
            entry.state.load_active_scenes(&mut self.event_outboxes);
        }

        while !self.should_quit {
            for event in sdl3::events::poll() {
                trace!("SDL event: {:?}", event);

                match event {
                    SdlEvent::Quit => self.quit(),
                    _ => {}
                }
            }

            self.drain_app_events();

            if self.should_quit {
                break;
            }

            let now = Instant::now();
            self.clock.advance(now);

            while self.clock.should_tick() {
                for entry in self.windows.values_mut() {
                    entry.state.sync_time(&self.clock, &entry.pacer);
                    entry
                        .state
                        .update_active_scenes(UpdatePhase::Fixed, &mut self.event_outboxes);
                }

                self.clock.consume();
            }

            let mut rendered = false;
            let now_after_tick = Instant::now();

            for entry in self.windows.values_mut() {
                if !entry.pacer.due(now) {
                    continue;
                }

                rendered = true;

                entry.pacer.record(now_after_tick);
                entry.state.sync_time(&self.clock, &entry.pacer);
                entry
                    .state
                    .update_active_scenes(UpdatePhase::Unrestrained, &mut self.event_outboxes);
                entry.state.draw_active_scenes(&mut self.event_outboxes);

                entry.sdl_window.clear(Color::RED);
            }

            if rendered {
                // Roll input
            }

            let now_after_render = Instant::now();
            let tick = self.clock.next_tick();

            let deadline = self
                .windows
                .values()
                .map(|e| {
                    e.pacer
                        .deadline()
                        .unwrap_or(now_after_render + e.pacer.idle_backoff())
                })
                .fold(tick, Instant::min);

            sleep_precise_until(deadline);
        }

        info!("App lifecycle ended, exiting.");
    }
}

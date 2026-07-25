use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::time::Instant;

use utils::FastHashMap;
use utils::SleepTimer;
use utils::WindowId;

use crate::WindowBuilder;
use crate::render::layer::{Layer, LayerCameras};
use crate::render::packet::FramePacket;
use crate::render::view::View;
use crate::scene::SceneRegistry;
use crate::window;
use crate::window::SdlEvent;
use crate::window::SdlWindowEvent;
use crate::window::WindowHandle;
use crate::window::context::WindowContext;
use crate::window::input::Input;
use crate::window::time::Time;

pub struct Simulation {
    views: Vec<View>,
    context: WindowContext,
    scenes: SceneRegistry,
    active_scenes: Vec<String>,
}

impl Deref for Simulation {
    type Target = WindowContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for Simulation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl Simulation {
    fn tick(&mut self) {
        for label in &self.active_scenes {
            let scene = self.scenes.get_mut(label);
            let (ctx, mut s) = self.context.split_scene();
            scene.update(ctx, &mut s);
        }
    }

    fn draw(&mut self) {
        let viewport = self.context.window.size();

        self.context.packet.clear();

        for l in Layer::ALL {
            self.context.cameras[l].update(viewport);
            self.context.packet[l].camera = self.context.cameras[l].packet();
        }

        for label in &self.active_scenes {
            let scene = self.scenes.get_mut(label);
            let (ctx, mut draw) = self.context.split_draw();

            scene.draw(ctx, &mut draw);
        }

        self.context.window.submit(self.context.packet.clone());
    }
}

pub struct SimulationRunner {
    simulations: FastHashMap<WindowId, Simulation>,
    event_receiver: Receiver<SdlEvent>,
    sleeper: SleepTimer,
}

impl SimulationRunner {
    const MAX_CATCH_UP: u32 = 5;

    pub fn new(e: Receiver<SdlEvent>) -> Self {
        Self {
            simulations: FastHashMap::default(),
            event_receiver: e,
            sleeper: SleepTimer::new(),
        }
    }

    pub fn insert_window(&mut self, window: WindowHandle, b: WindowBuilder) {
        let window_id = window.id();
        let simulation = Simulation {
            views: Vec::new(),
            context: WindowContext {
                window,
                time: Time::new(),
                input: Input::default(),
                packet: FramePacket::default(),
                cameras: LayerCameras::new(b.size),
            },
            scenes: SceneRegistry::new(b.scenes),
            active_scenes: b.active_scenes,
        };

        self.simulations.insert(window_id, simulation);
    }

    fn get_sim_mut(&mut self, id: &WindowId) -> &mut Simulation {
        self.simulations
            .get_mut(id)
            .expect("Failed to get simulation")
    }

    fn handle_event(&mut self, event: SdlEvent) {
        match event {
            SdlEvent::Window {
                window_id,
                win_event,
                ..
            } => match win_event {
                SdlWindowEvent::CloseRequested => {
                    self.simulations.remove(&window_id);
                }

                SdlWindowEvent::Resized(x, y) => {}

                _ => {}
            },

            SdlEvent::KeyDown {
                window_id,
                scancode: Some(code),
                repeat,
                ..
            } => {
                let sim = self.get_sim_mut(&window_id);

                if !repeat {
                    sim.input.k_pressed.insert(code);
                }

                sim.input.k_down.insert(code);
            }

            SdlEvent::KeyUp {
                window_id,
                scancode: Some(code),
                ..
            } => {
                let sim = self.get_sim_mut(&window_id);

                sim.input.k_down.remove(&code);
                sim.input.k_released.insert(code);
            }

            SdlEvent::MouseButtonDown {
                window_id,
                mouse_btn,
                clicks,
                ..
            } => {
                let sim = self.get_sim_mut(&window_id);

                if clicks == 1 {
                    sim.input.m_pressed.insert(mouse_btn);
                }

                sim.input.m_down.insert(mouse_btn);
            }

            SdlEvent::MouseButtonUp {
                window_id,
                mouse_btn,
                ..
            } => {
                let sim = self.get_sim_mut(&window_id);

                sim.input.m_down.remove(&mouse_btn);
                sim.input.m_released.insert(mouse_btn);
            }

            SdlEvent::MouseMotion {
                window_id,
                x,
                y,
                xrel,
                yrel,
                ..
            } => {
                let sim = self.get_sim_mut(&window_id);

                sim.input.m_pos.set([x, y]);
                sim.input.m_delta.set([xrel, yrel]);
            }

            _ => {}
        }
    }

    pub fn run(mut self) {
        'sim: loop {
            while let Ok(event) = self.event_receiver.try_recv() {
                self.handle_event(event);
            }

            if self.simulations.is_empty() {
                break 'sim;
            }

            for sim in self.simulations.values_mut() {
                sim.time.advance();

                let mut steps = 0;

                while sim.time.should_tick() && steps < Self::MAX_CATCH_UP {
                    let start = Instant::now();
                    sim.time.consume(start);
                    steps += 1;
                }

                if steps == Self::MAX_CATCH_UP {
                    sim.time.drop_backlog();
                }

                if steps > 0 {
                    sim.tick();
                    sim.draw();
                }
            }

            let sleep = self
                .simulations
                .values_mut()
                .map(|s| {
                    s.input.flush();
                    s.time.until_next_tick()
                })
                .min()
                .unwrap_or(Duration::from_millis(1));

            self.sleeper.sleep(sleep);
        }
    }
}

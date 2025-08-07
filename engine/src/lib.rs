mod context;
mod scene;
mod time;
mod window;

pub use context::Context;
pub use scene::{LoadControlFlow, Scene};
pub use time::Time;
pub use window::Window;

use err::EngineError;
use math::Size;
use sdl2::event::Event;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct SceneManager {
    scenes: HashMap<String, Box<dyn Scene>>,
    current_scene: Option<String>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            current_scene: None,
        }
    }

    pub fn add_scene(&mut self, name: String, scene: Box<dyn Scene>) {
        self.scenes.insert(name, scene);
    }

    pub fn switch_scene(&mut self, name: &str) -> Result<(), String> {
        if self.scenes.contains_key(name) {
            self.current_scene = Some(name.to_string());
            Ok(())
        } else {
            Err(format!("Scene '{}' not found", name))
        }
    }

    pub fn current_scene_mut(&mut self) -> Option<&mut Box<dyn Scene>> {
        if let Some(ref current_name) = self.current_scene {
            self.scenes.get_mut(current_name)
        } else {
            None
        }
    }
}

pub struct App {
    sdl: sdl2::Sdl,
    _video: sdl2::VideoSubsystem,
    window: Option<Window>,
    scene_manager: SceneManager,
    initial_scene: Option<String>,
}

impl App {
    pub fn new<T: AsRef<str>, S: Into<Size<u32>>>(title: T, size: S) -> Result<Self, EngineError> {
        let size: Size<u32> = size.into();

        let sdl = sdl2::init().map_err(|e| EngineError::SdlInit(e.to_string()))?;
        let video = sdl
            .video()
            .map_err(|e| EngineError::VideoInit(e.to_string()))?;

        let window = Window::new(&video, title, size.width, size.height)?;

        Ok(Self {
            sdl,
            _video: video,
            window: Some(window),
            scene_manager: SceneManager::new(),
            initial_scene: None,
        })
    }

    pub fn with_scene<N: Into<String>, S: Scene + 'static>(mut self, name: N, scene: S) -> Self {
        self.scene_manager.add_scene(name.into(), Box::new(scene));
        self
    }

    pub fn with_initial_scene<S: Into<String>>(mut self, name: S) -> Self {
        self.initial_scene = Some(name.into());
        self
    }

    pub fn run(mut self) -> Result<(), EngineError> {
        let window = self.window.take().expect("cannot fail");
        let mut context = Context::new(window);

        let mut event_pump = self
            .sdl
            .event_pump()
            .map_err(|e| EngineError::EventPumpCreation(e.to_string()))?;

        // Set the initial scene if specified
        if let Some(scene) = &self.initial_scene {
            if let Err(e) = self.scene_manager.switch_scene(&scene) {
                return Err(EngineError::SceneNotFound(format!(
                    "Initial scene '{}' not found: {}",
                    scene, e
                )));
            }
        }

        let mut t0 = Instant::now();
        let mut acc = 0.0;
        let mut should_load = true;

        while context.running() {
            let t1 = Instant::now();
            let dt = t1.duration_since(t0).as_secs_f64();

            t0 = t1;
            acc += dt;

            context.time.update(dt);

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        context.set_running(false);
                    }

                    _ => {}
                }
            }

            while acc >= context.time.update_step() {
                // scene.fixed_update(&mut context);
                acc -= context.time.update_step();
            }

            if let Some(scene) = self.scene_manager.current_scene_mut() {
                if should_load {
                    if let Err((flow, e)) = scene.load(&mut context) {
                        match flow {
                            LoadControlFlow::Ignore => {}
                            LoadControlFlow::Throw => {
                                return Err(EngineError::SceneNotFound(e));
                            }
                        }
                    }

                    should_load = false;
                }

                scene.update(&mut context);
                scene.render(&mut context);
            }

            context.window.present();

            let frame_time = t1.elapsed().as_secs_f64();

            if frame_time < context.time.render_step() {
                let sleep_duration = context.time.render_step() - frame_time;
                spin_sleep::sleep(Duration::from_secs_f64(sleep_duration));
            }

            if let Some(scene) = context.take_scene_change_request() {
                if let Err(e) = self.scene_manager.switch_scene(&scene) {
                    eprintln!("Failed to switch scene: {}", e);
                } else {
                    should_load = true;
                }
            }
        }

        Ok(())
    }
}

use std::any::Any;
use std::sync::mpsc::Receiver;

use assets::AssetServer;
use imgui::ActiveImgui;
use imgui::SharedImgui;
use logging::info;
use renderer::Renderer;
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;

use crate::AppEvent;
use crate::Scene;
use crate::context::WindowContext;
use crate::monitors::Monitor;
use crate::scene::SceneCommand;
use crate::scene::Scenes;
use crate::window::Window;

pub struct WindowState {
    context: WindowContext,
    events: Receiver<AppEvent>,
    scenes: Scenes,
    active_scenes: Vec<String>,
}

impl WindowState {
    pub fn new(
        events: Receiver<AppEvent>,
        window: Window,
        assets: AssetServer,
        renderer: Renderer,
        scenes: Scenes,
        active_scenes: Vec<String>,
        monitors: Vec<Monitor>,
        imgui: SharedImgui,
    ) -> Self {
        let mut state = Self {
            context: WindowContext::new(window, assets, renderer, monitors, imgui),
            events,
            scenes,
            active_scenes: Vec::new(),
        };

        for label in active_scenes {
            state.activate_scene(label, None);
        }

        state
    }

    fn register_scene(&mut self, label: String, scene: Box<dyn Scene>) {
        self.scenes.register(label, Box::new(move |_| scene));
    }

    fn activate_scene(&mut self, label: String, user_data: Option<Box<dyn Any>>) {
        if self.active_scenes.contains(&label) {
            return;
        }

        self.scenes.build(&label, self.context.as_ref_mut());

        if let Some(scene) = self.scenes.get_mut(&label) {
            scene.load(self.context.as_ref_mut());

            info!("Scene '{}' loaded with user data = {:?}", label, user_data);

            if let Some(user_data) = user_data {
                scene.loaded_with(self.context.as_ref_mut(), user_data);
            }
        }

        self.active_scenes.push(label);
    }

    fn deactivate_scene(&mut self, label: &str) {
        self.active_scenes.retain(|s| s != label);
    }

    pub fn start_loop(mut self) {
        let mut should_close = false;

        while !should_close {
            for event in self.events.try_iter() {
                match event {
                    AppEvent::WindowEvent(WindowEvent::CloseRequested) => {
                        should_close = true;
                        break;
                    }

                    e => {
                        self.context.handle_event(&e);
                        self.context.handle_event_for_imgui(&e);
                    }
                }
            }

            self.context.update_imgui_time();

            for label in &self.active_scenes {
                if let Some(scene) = self.scenes.get_mut(label) {
                    scene.update(self.context.as_ref_mut());
                }
            }

            while let Some(tick_start) = self.context.time.next_tick() {
                self.context.time.do_tick(tick_start);

                for label in &self.active_scenes {
                    if let Some(scene) = self.scenes.get_mut(label) {
                        scene.fixed_update(self.context.as_ref_mut());
                    }
                }
            }

            if self.context.input.key_pressed(&KeyCode::BracketRight) {
                self.context.show_console = !self.context.show_console;
            }

            for label in &self.active_scenes {
                let show_console = self.context.show_console;
                let (ctx, mut draw) = self.context.split();

                if let Some(scene) = self.scenes.get_mut(label) {
                    scene.draw(ctx, &mut draw);
                }

                if show_console {
                    draw.console();
                }
            }

            {
                let mut imgui = ActiveImgui::new(&self.context.imgui, self.context.window.id());
                self.context
                    .renderer
                    ._present(&self.context.assets._guard(), &mut imgui);
            }

            for command in self.context.scenes.drain_collect() {
                match command {
                    SceneCommand::Register { label, scene } => {
                        self.register_scene(label, scene);
                    }
                    SceneCommand::Activate { label, user_data } => {
                        self.activate_scene(label, user_data);
                    }
                    SceneCommand::Deactivate { label } => {
                        self.deactivate_scene(&label);
                    }
                }
            }

            self.context.input.flush();
            self.context.window.request_redraw();
            self.context.time.wait_for_next_frame();
        }
    }
}

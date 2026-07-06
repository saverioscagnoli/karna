use std::sync::mpsc::Receiver;

use assets::AssetServer;
use imgui::ActiveImgui;
use imgui::SharedImgui;
use logging::info;
use renderer::Renderer;
use winit::event::WindowEvent;

use crate::AppEvent;
use crate::context::WindowContext;
use crate::monitors::Monitor;
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
            if let Some(scene) = state.scenes.get_mut(&label) {
                scene.load(state.context.as_ref_mut());
                info!("Scene '{}' loaded", label);
            }

            state.active_scenes.push(label);
        }

        state
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

                    e => self.context.handle_event(e),
                }
            }

            self.context.update_imgui();

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

            for label in &self.active_scenes {
                let (ctx, mut draw) = self.context.split();

                if let Some(scene) = self.scenes.get_mut(label) {
                    scene.draw(ctx, &mut draw);
                }
            }

            {
                let mut imgui = ActiveImgui::new(&self.context.imgui, self.context.window.id());
                self.context
                    .renderer
                    ._present(&self.context.assets._guard(), &mut imgui);
            }

            let to_activate: Vec<_> = self.context.scenes.pending_activate.drain(..).collect();
            let to_deactivate: Vec<_> = self.context.scenes.pending_deactivate.drain(..).collect();

            for label in to_activate {
                if !self.active_scenes.contains(&label) {
                    if let Some(scene) = self.scenes.get_mut(&label) {
                        scene.load(self.context.as_ref_mut());
                        info!("Scene '{}' loaded", label);
                    }

                    self.active_scenes.push(label);
                }
            }

            for label in to_deactivate {
                self.active_scenes.retain(|s| s != &label);
            }

            self.context.input.flush();
            self.context.window.request_redraw();
            self.context.time.wait_for_next_frame();
        }
    }
}

use assets::AssetServer;
use assets::AssetServerGuard;
use renderer::Draw;
use renderer::Renderer;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::input::Input;
use crate::monitors::Monitor;
use crate::monitors::Monitors;
use crate::scene::SceneManager;
use crate::time::Time;
use crate::window::Window;

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub assets: AssetServer,
    pub scenes: SceneManager,
    pub renderer: Renderer,
    pub imgui: imgui::Context,
    pub monitors: Monitors,
}

impl WindowContext {
    pub fn new(
        window: Window,
        assets: AssetServer,
        renderer: Renderer,
        monitors: Vec<Monitor>,
        imgui: imgui::Context,
    ) -> Self {
        // Arc::clone
        let winit_window = window.inner.clone();

        Self {
            window,
            time: Time::new(),
            input: Input::new(),
            assets,
            scenes: SceneManager::new(),
            renderer,
            monitors: Monitors::new(winit_window, monitors),
            imgui,
        }
    }

    pub fn as_ref_mut<'ctx>(&'ctx mut self) -> ContextRefMut<'ctx> {
        ContextRefMut {
            window: &self.window,
            time: &mut self.time,
            input: &mut self.input,
            assets: &self.assets,
            scenes: &mut self.scenes,
            render: &mut self.renderer,
            monitors: &self.monitors,
        }
    }

    pub fn split<'ctx>(&'ctx mut self) -> (ContextRef<'ctx>, Draw<'ctx>) {
        let ctx = ContextRef {
            window: &self.window,
            time: &self.time,
            input: &self.input,
            assets: self.assets._guard(),
            scenes: &self.scenes,
            monitors: &self.monitors,
        };

        let draw = Draw::_new(&mut self.renderer, self.assets._guard(), &mut self.imgui);

        (ctx, draw)
    }

    pub fn update_imgui(&mut self) {
        let io = self.imgui.io_mut();
        io.delta_time = self.time.delta();
        io.display_size = self.window.size().as_f32().into();

        io.key_ctrl = self.input.held_keys.contains(&KeyCode::ControlLeft)
            || self.input.held_keys.contains(&KeyCode::ControlRight);
        io.key_shift = self.input.held_keys.contains(&KeyCode::ShiftLeft)
            || self.input.held_keys.contains(&KeyCode::ShiftRight);
        io.key_alt = self.input.held_keys.contains(&KeyCode::AltLeft)
            || self.input.held_keys.contains(&KeyCode::AltRight);
        io.key_super = self.input.held_keys.contains(&KeyCode::SuperLeft)
            || self.input.held_keys.contains(&KeyCode::SuperRight);

        //for key in &self.input.held_keys {
        //    let code = imgui_key_index(*key);
        //    if code < io.keys_down.len() {
        //        io.keys_down[code] = true;
        //    }
        //}

        io.mouse_pos = self.input.mouse_position.into();
        io.mouse_delta = self.input.mouse_delta.into();
        io.mouse_wheel = self.input.wheel_delta.y;
        io.mouse_wheel_h = self.input.wheel_delta.x;
        io.mouse_down = [
            self.input.mouse_held(&MouseButton::Left),
            self.input.mouse_held(&MouseButton::Right),
            self.input.mouse_held(&MouseButton::Middle),
            false,
            false,
        ];
    }
}

pub struct ContextRefMut<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx mut Time,
    pub input: &'ctx mut Input,
    pub assets: &'ctx AssetServer,
    pub scenes: &'ctx mut SceneManager,
    pub render: &'ctx mut Renderer,
    pub monitors: &'ctx Monitors,
}

pub struct ContextRef<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx Time,
    pub input: &'ctx Input,
    pub assets: AssetServerGuard<'ctx>,
    pub scenes: &'ctx SceneManager,
    pub monitors: &'ctx Monitors,
}

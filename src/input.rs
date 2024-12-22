use std::time::Duration;

use hashbrown::HashSet;
pub use sdl2::controller::Button;
use sdl2::controller::GameController;
pub use sdl2::keyboard::Keycode as Key;
pub use sdl2::mouse::MouseButton as Mouse;
use sdl2::{GameControllerSubsystem, Sdl};

use crate::math::Vec2;

pub struct Input {
    controller_sys: GameControllerSubsystem,

    pub(crate) keys: HashSet<Key>,
    pub(crate) pressed_keys: HashSet<Key>,

    pub(crate) mouse_buttons: HashSet<Mouse>,
    pub(crate) clicked_mouse_buttons: HashSet<Mouse>,
    pub(crate) mouse_position: Vec2,

    controller: Option<GameController>,
    pub(crate) left_stick: Vec2,
    pub(crate) right_stick: Vec2,
    pub(crate) left_trigger: f32,
    pub(crate) right_trigger: f32,
    pub(crate) buttons: HashSet<Button>,
    pub(crate) pressed_buttons: HashSet<Button>,
}

impl Input {
    pub(crate) fn new(sdl: &Sdl) -> Self {
        let controller_sys = sdl.game_controller().unwrap();

        Self {
            controller_sys,
            keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            mouse_buttons: HashSet::new(),
            clicked_mouse_buttons: HashSet::new(),
            mouse_position: Vec2::ZERO,
            controller: None,
            left_stick: Vec2::ZERO,
            right_stick: Vec2::ZERO,
            left_trigger: 0.0,
            right_trigger: 0.0,
            buttons: HashSet::new(),
            pressed_buttons: HashSet::new(),
        }
    }

    pub(crate) fn scan_controllers(&mut self) {
        let n = self.controller_sys.num_joysticks().unwrap();

        let controller = (0..n).find_map(|i| {
            if !self.controller_sys.is_game_controller(i) {
                return None;
            }

            match self.controller_sys.open(i) {
                Ok(c) => {
                    println!("Controller {} connected.", c.name());
                    Some(c)
                }
                Err(_) => None,
            }
        });

        if controller.is_none() {
            println!("No controller found.");
        }

        self.controller = controller;
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn mouse_down(&self, button: Mouse) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn mouse_clicked(&self, button: Mouse) -> bool {
        self.clicked_mouse_buttons.contains(&button)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn left_stick(&self) -> Vec2 {
        self.left_stick
    }

    pub fn right_stick(&self) -> Vec2 {
        self.right_stick
    }

    pub fn left_trigger(&self) -> f32 {
        self.left_trigger
    }

    pub fn right_trigger(&self) -> f32 {
        self.right_trigger
    }

    pub fn button_down(&self, button: Button) -> bool {
        self.buttons.contains(&button)
    }

    pub fn button_pressed(&self, button: Button) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn rumble(&mut self, left: f32, right: f32, duration: Duration) {
        let left = left.clamp(0.0, 1.0);
        let right = right.clamp(0.0, 1.0);

        // Map the range [0.0, 1.0] to [0, u16::MAX]
        let left_mapped = (left * u16::MAX as f32) as u16;
        let right_mapped = (right * u16::MAX as f32) as u16;

        if let Some(controller) = &mut self.controller {
            controller
                .set_rumble(left_mapped, right_mapped, duration.as_millis() as u32)
                .unwrap();
        }
    }

    pub(crate) fn flush(&mut self) {
        self.pressed_keys.clear();
        self.clicked_mouse_buttons.clear();
        self.pressed_buttons.clear();
    }
}

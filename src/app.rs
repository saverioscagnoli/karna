use crate::{
    math::Size,
    traits::{Draw, Load, Update},
    Context,
};
use rodio::Sample;
use sdl2::{controller::Axis, event::Event, pixels::Color};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const ONE_SEC: Duration = Duration::from_secs(1);

/// App flow:
///
/// 1. App::new gets called with a title and a size
/// 2. Context is created
///   - Sdl is initialized
///   - Video subsystem is initialized
///   - Window is created
///   - Renderer is created
///       - Set the oncelock with the texture creator, to store textures as 'static
///       - Texture atlas is created and stored in the renderer
///   - Time helper is created
///   - Input helper is created
///   - Audio helper is created
///   - Ui helper is created
/// 3. App::run is called by the user
///   - Game loop starts
///   - Listens for events

pub struct App {
    ctx: Context,
}

impl App {
    pub fn new<T: ToString, S: Into<Size>>(title: T, size: S) -> Result<Self, String> {
        let ctx = Context::new(title, size)?;

        Ok(Self { ctx })
    }

    pub fn run<S: Load + Update + Draw>(&mut self, state: &mut S) {
        self.ctx.start();

        state.load(&mut self.ctx);

        self.ctx.window.show();

        let mut event_pump = self.ctx.sdl.sys.event_pump().unwrap();

        let mut t0 = Instant::now();
        let mut acc = 0.0;

        let mut times = VecDeque::new();

        while self.ctx.running {
            let t1 = Instant::now();
            let dt = t1.duration_since(t0).as_secs_f32();
            t0 = t1;

            acc += dt;

            self.ctx.time.delta = dt;

            while times.len() > 0 && times[0] <= t1 - ONE_SEC {
                times.pop_front();
            }

            times.push_back(t1);

            self.ctx.time.fps = times.len() as u32;

            self.handle_events(&mut event_pump);

            while acc >= self.ctx.time.tick_step {
                state.fixed_update(&mut self.ctx);
                acc -= self.ctx.time.tick_step;
            }

            state.update(&mut self.ctx);

            self.ctx.render.clear();
            state.draw(&mut self.ctx);

            self.ctx.render.set_color(Color::BLACK);

            self.ctx.render.present();

            self.ctx.input.flush();

            let t2 = t1.elapsed().as_secs_f32();
            let remaining = self.ctx.time.render_step - t2;

            if remaining > 0.0 {
                spin_sleep::sleep(Duration::from_secs_f32(remaining));
            }
        }
    }

    fn handle_events(&mut self, event_pump: &mut sdl2::EventPump) {
        let ctx = &mut self.ctx;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => ctx.stop(),

                // Keyboard events
                Event::KeyDown {
                    keycode: Some(key),
                    repeat,
                    ..
                } => {
                    if !ctx.input.key_down(key) {
                        ctx.input.keys.push(key);
                    }

                    if !repeat && !ctx.input.key_pressed(key) {
                        ctx.input.pressed_keys.push(key);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    ctx.input.keys.retain(|&k| k != key);
                }

                // Mouse events
                Event::MouseButtonDown { mouse_btn, .. } => {
                    if !ctx.input.mouse_down(mouse_btn) {
                        ctx.input.mouse_buttons.push(mouse_btn);
                    }

                    if !ctx.input.mouse_clicked(mouse_btn) {
                        ctx.input.clicked_mouse_buttons.push(mouse_btn);
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    ctx.input.mouse_buttons.retain(|&b| b != mouse_btn);
                }

                Event::MouseMotion { x, y, .. } => {
                    ctx.input.mouse_position.set(x, y);
                }

                // Rescan controllers every time a controller is added or removed
                Event::ControllerDeviceAdded { .. } => ctx.input.scan_controllers(),
                Event::ControllerDeviceRemoved { .. } => ctx.input.scan_controllers(),

                // Controller events
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let mut value = value.to_f32();

                    // These small corrections are necessary because the controller
                    // reports values slightly above 1.0 and slightly below 0.0
                    // idk if this is due to sdl2 or not, but i checked the cotroller
                    // values and they are fine, so i think it's sdl2
                    match axis {
                        Axis::LeftX => {
                            if value > 0.99 {
                                value = 1.0;
                            }

                            ctx.input.left_stick.x = value;
                        }
                        Axis::LeftY => {
                            if value > 0.99 {
                                value = 1.0;
                            }

                            if value > -0.01 && value < 0.0 {
                                value = 0.0;
                            }

                            ctx.input.left_stick.y = value;
                        }
                        Axis::RightX => {
                            if value > 0.99 {
                                value = 1.0;
                            }

                            ctx.input.right_stick.x = value;
                        }
                        Axis::RightY => {
                            if value > 0.99 {
                                value = 1.0;
                            }

                            if value > -0.01 && value < 0.0 {
                                value = 0.0;
                            }

                            ctx.input.right_stick.y = value;
                        }
                        Axis::TriggerLeft => {
                            ctx.input.left_trigger = {
                                if value > 0.99 {
                                    1.0
                                } else {
                                    value
                                }
                            }
                        }
                        Axis::TriggerRight => {
                            ctx.input.right_trigger = {
                                if value > 0.99 {
                                    1.0
                                } else {
                                    value
                                }
                            }
                        }
                    }
                }

                Event::ControllerButtonDown { button, .. } => {
                    if !ctx.input.button_down(button) {
                        ctx.input.buttons.push(button);
                    }

                    if !ctx.input.button_pressed(button) {
                        ctx.input.pressed_buttons.push(button);
                    }
                }

                Event::ControllerButtonUp { button, .. } => {
                    ctx.input.buttons.retain(|&b| b != button);
                }

                _ => {}
            }
        }
    }
}

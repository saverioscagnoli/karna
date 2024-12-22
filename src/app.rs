use crate::{math::Size, traits::Scene, Context};
use hashbrown::HashMap;
use rodio::Sample;
use sdl2::{
    controller::Axis,
    event::{Event, WindowEvent},
};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const ONE_SEC: Duration = Duration::from_secs(1);

pub struct App {
    ctx: Context,
    scenes: HashMap<String, Box<dyn Scene>>,
}

impl App {
    pub fn window<T: ToString, S: Into<Size>>(title: T, size: S) -> Self {
        let size: Size = size.into();

        Self {
            ctx: Context::new(&title.to_string(), size.width, size.height),
            scenes: HashMap::new(),
        }
    }

    pub fn run<S: Scene + 'static>(&mut self, first_scene: S) {
        self.ctx.start();

        self.scenes
            .insert("first".to_string(), Box::new(first_scene));

        let first_scene = self.scenes.get_mut("first").unwrap();

        first_scene.load(&mut self.ctx);

        let mut t0 = Instant::now();
        let mut acc = 0.0;

        let mut times = VecDeque::new();

        while self.ctx.running {
            let t1 = Instant::now();
            let dt = t1.duration_since(t0).as_secs_f32();

            self.ctx.time.delta = dt;

            t0 = t1;
            acc += dt;

            times.push_back(t1);
            Self::handle_events(&mut self.ctx);

            while acc >= self.ctx.time.tick_step {
                first_scene.fixed_update(&mut self.ctx);
                acc -= self.ctx.time.tick_step;
            }

            while times.len() > 0 && times[0] <= t1 - ONE_SEC {
                times.pop_front();
            }

            self.ctx.time.fps = times.len() as u32;

            first_scene.update(&mut self.ctx);

            self.ctx.render.clear();

            first_scene.draw(&mut self.ctx);

            self.ctx.render.present();
            self.ctx.window.swap_buffers();

            self.ctx.input.flush();

            // let t2 = t1.elapsed().as_secs_f32();
            // let remaining = self.ctx.time.render_step - t2;

            // if remaining > 0.0 {
            //     spin_sleep::sleep(Duration::from_secs_f32(remaining));
            // }
        }
    }

    fn handle_events(ctx: &mut Context) {
        for event in ctx.sdl.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => ctx.running = false,
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(w, h) => {
                        unsafe { gl::Viewport(0, 0, w, h) };
                    }

                    WindowEvent::SizeChanged(w, h) => {
                        unsafe { gl::Viewport(0, 0, w, h) };
                    }

                    _ => {}
                },

                Event::KeyDown {
                    repeat,
                    keycode: Some(code),
                    ..
                } => {
                    ctx.input.keys.insert(code);
                    if !repeat {
                        ctx.input.pressed_keys.insert(code);
                    }
                }

                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => {
                    ctx.input.keys.remove(&code);
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    ctx.input.mouse_buttons.insert(mouse_btn);
                    ctx.input.clicked_mouse_buttons.insert(mouse_btn);
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    ctx.input.mouse_buttons.remove(&mouse_btn);
                }

                Event::MouseMotion { x, y, .. } => ctx.input.mouse_position.set(x as f32, y as f32),

                Event::ControllerDeviceAdded { .. } | Event::ControllerDeviceRemoved { .. } => {
                    ctx.input.scan_controllers();
                }

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
                    ctx.input.buttons.insert(button);
                    ctx.input.pressed_buttons.insert(button);
                }

                Event::ControllerButtonUp { button, .. } => {
                    ctx.input.buttons.remove(&button);
                }

                _ => {}
            }
        }
    }
}

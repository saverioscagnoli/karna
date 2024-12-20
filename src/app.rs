use crate::{math::Size, traits::Scene, Context};
use gl::types::{GLint, GLsizei, GLsizeiptr};
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

    fn setup_buffers(&mut self) -> ([f32; 16], [u32; 6], (u32, u32, u32)) {
        let vertices: [f32; 16] = [
            // Positions    // Texture Coords
            -1.0, -1.0, 0.0, 1.0, // Bottom-left
            1.0, -1.0, 1.0, 1.0, // Bottom-right
            1.0, 1.0, 1.0, 0.0, // Top-right
            -1.0, 1.0, 0.0, 0.0, // Top-left
        ];

        let indices: [u32; 6] = [
            0, 1, 2, // First triangle
            2, 3, 0, // Second triangle
        ];

        let vs = include_str!("../assets/vs.glsl");
        let fs = include_str!("../assets/fs.glsl");

        self.ctx.render.load_shader("default", vs, fs);

        let (vbo, vao, ebo) = (0, 0, 0);

        (vertices, indices, (vbo, vao, ebo))
    }

    pub fn run<S: Scene + 'static>(&mut self, first_scene: S) {
        self.ctx.start();
        let (vertices, indices, (mut vbo, mut vao, mut ebo)) = self.setup_buffers();

        unsafe {
            // Generate and bind Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Generate and bind Vertex Buffer Object
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Generate and bind Element Buffer Object
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Position attribute
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                0 as *const _,
            );
            gl::EnableVertexAttribArray(0);

            // Texture coordinate attribute
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        unsafe {
            gl::GenTextures(1, &mut self.ctx.render.texture_id);
            gl::BindTexture(gl::TEXTURE_2D, self.ctx.render.texture_id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        self.scenes
            .insert("first".to_string(), Box::new(first_scene));

        self.ctx.render.set_shader("default");

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

            self.ctx.render.gl_present_surface();
            self.ctx.render.draw_quad(vao, indices.len());
            self.ctx.window.swap_buffers();

            self.ctx.input.flush();

            let t2 = t1.elapsed().as_secs_f32();
            let remaining = self.ctx.time.render_step - t2;

            if remaining > 0.0 {
                spin_sleep::sleep(Duration::from_secs_f32(remaining));
            }
        }

        self.clean(vbo, vao, ebo);
    }

    fn handle_events(ctx: &mut Context) {
        for event in ctx.sdl.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => ctx.running = false,
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(w, h) => ctx.render.update((w, h).into()),

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

                Event::MouseMotion { x, y, .. } => ctx.input.mouse_position.set(x, y),

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

    fn clean(&self, vbo: u32, vao: u32, ebo: u32) {
        unsafe {
            gl::DeleteTextures(1, &self.ctx.render.texture_id);
            gl::DeleteBuffers(1, &vbo);
            gl::DeleteBuffers(1, &ebo);
            gl::DeleteVertexArrays(1, &vao);

            self.ctx.render.clean_shaders();
        }
    }
}

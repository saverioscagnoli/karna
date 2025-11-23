use std::sync::Arc;

use renderer::{Color, OrthographicCamera, PerspectiveCamera, Renderer};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    mouse_grabbed: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            mouse_grabbed: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = PhysicalSize::new(1280, 720);
        let attributes = Window::default_attributes()
            .with_inner_size(size)
            .with_resizable(false);
        let window = event_loop.create_window(attributes).expect(":(");

        window.request_redraw();
        let window = Arc::new(window);

        let mut renderer = pollster::block_on(Renderer::new(window.clone())).unwrap();

        // let mut camera = PerspectiveCamera::new(
        //     renderer.state.device.clone(),
        //     [0.0, 0.0, 3.0].into(), // Move camera back so we can see objects at origin
        //     75.0_f32.to_radians(),  // Convert FOV to radians
        //     1280.0 / 720.0,
        //     0.1,
        //     1000.0,
        // );

        let mut camera = OrthographicCamera::new(
            renderer.state.device.clone(),
            [0.0, 0.0, 10.0].into(), // Move camera FORWARD on Z-axis so it can see Z=0
            0.0,                     // left
            1280.0,                  // right
            720.0,                   // bottom
            0.0,                     // top
            0.1,                     // near
            100.0,                   // far
        );

        // Set camera to look in -Z direction (into the scene)
        // Yaw of -90 degrees makes forward vector point in -Z direction
        //  camera.set_rotation(-90.0_f32.to_radians(), 0.0);

        renderer.set_camera(Box::new(camera));

        // Grab the cursor for FPS-style mouse look
        window.set_cursor_visible(false);
        let _ = window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked));

        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        self.mouse_grabbed = true;
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::MouseInput { button, state, .. } => {
                // Toggle mouse grab with right click
                if button == winit::event::MouseButton::Right
                    && state == winit::event::ElementState::Pressed
                {
                    if let Some(window) = &self.window {
                        self.mouse_grabbed = !self.mouse_grabbed;

                        if self.mouse_grabbed {
                            window.set_cursor_visible(false);
                            let _ = window
                                .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                .or_else(|_| {
                                    window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                });
                        } else {
                            window.set_cursor_visible(true);
                            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if !event.state.is_pressed() {
                    return;
                }

                match event.physical_key {
                    PhysicalKey::Code(code) => {
                        let Some(camera) = renderer.camera_mut() else {
                            return;
                        };

                        let move_speed = 0.1;
                        let rotation_speed = 2.0_f32.to_radians(); // 2 degrees per key press

                        match code {
                            KeyCode::KeyW => {
                                let forward = camera.forward();
                                camera.translate(forward * move_speed);
                            }
                            KeyCode::KeyS => {
                                let forward = camera.forward();
                                camera.translate(-forward * move_speed);
                            }
                            KeyCode::KeyA => {
                                let right = camera.right();
                                camera.translate(-right * move_speed);
                            }
                            KeyCode::KeyD => {
                                let right = camera.right();
                                camera.translate(right * move_speed);
                            }
                            KeyCode::ArrowLeft => {
                                camera.rotate(-rotation_speed, 0.0);
                            }
                            KeyCode::ArrowRight => {
                                camera.rotate(rotation_speed, 0.0);
                            }
                            KeyCode::ArrowUp => {
                                camera.rotate(0.0, rotation_speed);
                            }
                            KeyCode::ArrowDown => {
                                camera.rotate(0.0, -rotation_speed);
                            }
                            KeyCode::Escape => {
                                // Release mouse grab with Escape key
                                if let Some(window) = &self.window {
                                    self.mouse_grabbed = false;
                                    window.set_cursor_visible(true);
                                    let _ =
                                        window.set_cursor_grab(winit::window::CursorGrabMode::None);
                                }
                            }
                            _ => {}
                        }

                        println!("Camera position: {:?}", camera.position());

                        // Request redraw to see the camera movement
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                    _ => {}
                }
            }

            WindowEvent::RedrawRequested => {
                // Draw a 1x1 unit square at the origin in world space
                renderer.fill_rect_3d([0.0, 0.0, 20.0].into(), [10.0, 10.0].into());

                // Draw another square offset to the right
                renderer.set_draw_color(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
                renderer.fill_rect([2.0, 0.0].into(), [10.0, 10.0].into());

                // Reset color
                renderer.set_draw_color(Color::WHITE);

                renderer.end_frame();
            }

            _ => {}
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::with_user_event().build().expect(":(");

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(&mut self).expect(":(");
    }
}

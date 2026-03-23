use crossbeam_channel::Receiver;
use logging::warn;
use renderer::Renderer;
use winit::event::KeyEvent;
use winit::event::WindowEvent;
use winit::keyboard::PhysicalKey;

use crate::context::KeyState;
use crate::context::Window;
use crate::context::WindowContext;
use crate::scene::SceneManager;
use crate::scene::SceneMap;

pub struct WindowLifecycle {
    event_rx: Receiver<WindowEvent>,
    context: WindowContext,
    scene_manager: SceneManager,
}

impl WindowLifecycle {
    pub fn new(
        event_rx: Receiver<WindowEvent>,
        window: Window,
        window_surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
        scenes: SceneMap,
    ) -> Self {
        let render = Renderer::from_surface(window_surface, surface_config);

        Self {
            event_rx,
            context: WindowContext::new(window, render),
            scene_manager: SceneManager::new(scenes),
        }
    }

    pub fn game_loop(&mut self) {
        self.context.window.request_redraw();
        self.scene_manager.load(self.context.as_ref_mut());

        loop {
            // Drain all pending events, blocking only up to a short timeout
            // so we never get stuck waiting forever if the channel disconnects
            // or no events arrive.
            if self.drain_events() {
                return;
            }

            self.frame();
        }
    }

    /// Drains all available events from the channel.
    ///
    /// On the first call, blocks up to 1ms waiting for at least one event.
    /// After receiving one, drains the rest non-blocking with `try_recv`.
    ///
    /// Returns `true` if the game loop should exit (close requested or channel disconnected).
    fn drain_events(&mut self) -> bool {
        // Block briefly for the first event to avoid busy-spinning when idle.
        // A short timeout ensures we still render frames and detect channel disconnection.
        match self.event_rx.try_recv() {
            Ok(event) => {
                if self.handle_window_event(event) {
                    return true;
                }
            }
            _ => {}
        }

        // Drain any remaining buffered events without blocking.
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => {
                    if self.handle_window_event(event) {
                        return true;
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => return true,
            }
        }

        false
    }

    fn frame(&mut self) {
        self.context.time.frame_start();
        self.context.time.update();

        while let Some(tick_start) = self.context.time.next_tick() {
            self.scene_manager.fixed_update(self.context.as_ref_mut());
            self.context.time.do_tick(tick_start);
        }

        self.scene_manager.update(self.context.as_ref_mut());

        {
            let (ctx, mut draw) = self.context.split();

            self.scene_manager.draw(ctx, &mut draw);
        }

        self.context.render.present();
        self.context.time.frame_end();
        self.context.input.flush();

        self.context.time.wait_for_next_frame();
    }

    fn handle_window_event(&mut self, event: WindowEvent) -> bool {
        if let WindowEvent::CloseRequested = event {
            return true;
        }

        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                let KeyEvent {
                    physical_key,
                    state,
                    repeat,
                    ..
                } = key_event;

                match physical_key {
                    PhysicalKey::Code(code) => {
                        if state.is_pressed() {
                            if !repeat {
                                self.context
                                    .input
                                    .update_keystate(code, KeyState::Pressed, false);
                            }

                            self.context
                                .input
                                .update_keystate(code, KeyState::Held, false);
                        } else {
                            self.context
                                .input
                                .update_keystate(code, KeyState::Held, true);

                            self.context
                                .input
                                .update_keystate(code, KeyState::Released, false);
                        }
                    }

                    PhysicalKey::Unidentified(code) => {
                        warn!("Unidentified key code: {:?}", code);
                    }
                }
            }

            WindowEvent::Resized(view) => self.context.render.resize(view.into()),

            _ => {}
        }

        false
    }
}

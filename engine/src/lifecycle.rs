use assets::AssetServerGuard;
use crossbeam_channel::Receiver;
use renderer::Renderer;
use winit::event::ElementState;
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
    scenes: SceneManager,
    loaded: bool,
    focused: bool,
    pending_resize: Option<winit::dpi::PhysicalSize<u32>>,
}

impl WindowLifecycle {
    pub fn new(
        event_rx: Receiver<WindowEvent>,
        window: Window,
        window_surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
        scenes: SceneMap,
        assets: &AssetServerGuard,
    ) -> Self {
        let render = Renderer::from_surface(window_surface, surface_config, assets);

        Self {
            event_rx,
            context: WindowContext::new(window, render),
            scenes: SceneManager::new(scenes),
            loaded: false,
            focused: true,
            pending_resize: None,
        }
    }

    pub fn game_loop(&mut self) {
        // Load scenes once, on the window thread, before the first frame.
        if !self.loaded {
            let ctx = self.context.as_ref_mut();
            self.scenes.load(ctx);
            self.loaded = true;
        }

        loop {
            if self.drain_events() {
                return;
            }

            self.frame();
        }
    }

    /// Drains all available events from the channel.
    ///
    /// Returns `true` if the game loop should exit (close requested or channel disconnected).
    fn drain_events(&mut self) -> bool {
        // Drain all buffered events without blocking.
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
        if let Some(size) = self.pending_resize.take() {
            self.context
                .render
                .resize(math::Size::new(size.width, size.height));
        }

        self.context.time.frame_start();
        self.context.time.update();

        while let Some(tick_start) = self.context.time.next_tick() {
            self.context.time.do_tick(tick_start);

            let ctx = self.context.as_ref_mut();
            self.scenes.fixed_update(ctx);
        }

        {
            let ctx = self.context.as_ref_mut();
            self.scenes.update(ctx);
        }

        {
            let (ctx, mut draw) = self.context.split();
            self.scenes.draw(ctx, &mut draw);
        }

        self.context.render.present(&self.context.assets.guard());

        // Per-frame input cleanup. `Pressed` / `Released` should only live for a single frame.
        self.context.input.flush();

        self.context.time.frame_end();
        self.context.time.wait_for_next_frame();
    }

    fn handle_window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CloseRequested => return true,

            WindowEvent::Resized(physical_size) => {
                self.pending_resize = Some(physical_size);
            }

            // Track focus so you can decide how to treat input when unfocused.
            WindowEvent::Focused(focused) => {
                self.focused = focused;

                // When losing focus, clear held keys so they don't get "stuck".
                if !self.focused {
                    self.context.input.flush();
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                // Ignore key events while unfocused to avoid weird sticky transitions on Windows.
                if !self.focused {
                    return false;
                }

                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            // If it's not already held, mark as pressed for this frame.
                            if !self.context.input.key_held(&code) {
                                self.context
                                    .input
                                    .update_keystate(code, KeyState::Pressed, false);
                            }

                            self.context
                                .input
                                .update_keystate(code, KeyState::Held, false);
                        }

                        ElementState::Released => {
                            self.context
                                .input
                                .update_keystate(code, KeyState::Held, true);

                            self.context
                                .input
                                .update_keystate(code, KeyState::Released, false);
                        }
                    }
                }
            }

            _ => {}
        }

        false
    }
}

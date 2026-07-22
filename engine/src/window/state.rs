use std::mem;
use std::sync::mpsc::Receiver;

use logging::debug;
use sdl3::event::Event;
use sdl3::event::WindowEvent;

use crate::render::FramePacket;
use crate::render::FrameSubmission;
use crate::render::RenderLayers;
use crate::scene::SceneManagerCommand;
use crate::scene::SceneRegistry;
use crate::window::FrameSlot;
use crate::window::context::WindowContext;

pub struct WindowState {
    pub should_exit: bool,
    pub context: WindowContext,
    pub scenes: SceneRegistry,
    pub active_scenes: Vec<usize>,
    pub events: Receiver<Event>,
    pub shutdown: Receiver<()>,
    pub packet: FramePacket,

    /// Where finished frames are dropped for the main thread to render.
    pub frame_slot: FrameSlot,

    /// Geometry buffers coming back from the main thread after rendering,
    /// reused so vertex data isn't reallocated every frame.
    pub recycled: Receiver<RenderLayers>,

    /// SDL timestamp (ns) of the oldest input event not yet shipped in a frame.
    pub pending_input_timestamp: Option<u64>,
}

impl WindowState {
    fn init(&mut self) {
        for &index in &self.active_scenes {
            let scene = self.scenes.get_mut(index);
            let (mut ctx, mut view) = self.context.split_scene(&mut self.packet);

            scene.load(&mut ctx, &mut view);
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            &Event::Window { win_event, .. } => match win_event {
                WindowEvent::CloseRequested => self.should_exit = true,

                WindowEvent::Resized(w, h) => {
                    // The SDL GPU swapchain resizes itself; only the
                    // cached size needs updating.
                    let size = math::Size::new(w as u32, h as u32);

                    self.context.window.sync_size(size);
                    debug!("Resized window '{}' to {:?}", self.context.window.id, size);
                }

                _ => {}
            },

            &Event::KeyDown {
                timestamp,
                scancode: Some(c),
                repeat,
                ..
            } => {
                if !repeat {
                    self.pending_input_timestamp.get_or_insert(timestamp);
                    self.context.input.pressed_keys.insert(c);
                }

                self.context.input.held_keys.insert(c);
            }

            &Event::KeyUp {
                timestamp,
                scancode: Some(c),
                ..
            } => {
                self.pending_input_timestamp.get_or_insert(timestamp);

                self.context.input.held_keys.remove(&c);
                self.context.input.released_keys.insert(c);
            }

            &Event::MouseMotion { x, y, .. } => {
                self.context.input.mouse_position.set([x, y]);
            }

            &Event::MouseButtonDown { mouse_btn, .. } => {
                self.context.input.held_mouse_buttons.insert(mouse_btn);
            }

            &Event::MouseButtonUp { mouse_btn, .. } => {
                self.context.input.held_mouse_buttons.remove(&mouse_btn);
                self.context.input.released_mouse_buttons.insert(mouse_btn);
            }

            _ => {}
        }
    }

    fn handle_event_imgui(&mut self, event: &Event, io: &mut imgui::Io) {
        match event {
            &Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h) => {
                    io.set_display_size([w as f32, h as f32]);
                    io.set_display_framebuffer_scale([1.0, 1.0]);
                }

                _ => {}
            },

            &Event::KeyDown {
                scancode: Some(code),
                ..
            } => {
                if let Some(imgui_mod) = imgui::sdl_scancode_to_imgui_mod(code) {
                    io.add_key_event(imgui_mod, true);
                }

                if let Some(imgui_key) = imgui::sdl_scancode_to_imgui(code) {
                    io.add_key_event(imgui_key, true);
                }
            }

            &Event::KeyUp {
                scancode: Some(code),
                ..
            } => {
                if let Some(imgui_mod) = imgui::sdl_scancode_to_imgui_mod(code) {
                    io.add_key_event(imgui_mod, false);
                }

                if let Some(imgui_key) = imgui::sdl_scancode_to_imgui(code) {
                    io.add_key_event(imgui_key, false);
                }
            }

            &Event::MouseMotion { x, y, .. } => {
                io.add_mouse_pos_event([x, y]);
            }

            &Event::MouseWheel {
                x, y, direction, ..
            } => {
                let flip = match direction {
                    sdl3::mouse::MouseWheelDirection::Flipped => -1.0,
                    _ => 1.0,
                };

                io.add_mouse_wheel_event([x * flip, y * flip]);
            }

            Event::TextInput { text, .. } => {
                for ch in text.chars() {
                    io.add_input_character(ch);
                }
            }

            &Event::MouseButtonDown { mouse_btn, .. } => {
                if let Some(imgui_button) = imgui::sdl_mousebutton_to_imgui(mouse_btn) {
                    io.add_mouse_button_event(imgui_button, true);
                }
            }

            &Event::MouseButtonUp { mouse_btn, .. } => {
                if let Some(imgui_button) = imgui::sdl_mousebutton_to_imgui(mouse_btn) {
                    io.add_mouse_button_event(imgui_button, false);
                }
            }

            _ => {}
        }
    }

    fn drain_events(&mut self, io: &mut imgui::Io) {
        io.set_delta_time(self.context.time.delta());

        while let Ok(event) = self.events.try_recv() {
            self.handle_event(&event);
            self.handle_event_imgui(&event, io);
        }
    }

    fn drain_scene_commands(&mut self) {
        for command in self.context.scenes.buffer.drain(..) {
            match command {
                SceneManagerCommand::Activate(index, user_data) => {}
                SceneManagerCommand::Deactivate(index) => {}
                SceneManagerCommand::Pause(index) => {}
                SceneManagerCommand::Resume(index) => {}
            }
        }
    }

    fn frame(&mut self) {
        self.packet.viewport = *self.context.window.size();
        self.packet.input_timestamp = self.pending_input_timestamp.take();

        while let Some(tick_start) = self.context.time.next_tick() {
            for &index in &self.active_scenes {
                let scene = self.scenes.get_mut(index);
                let (mut ctx, mut view) = self.context.split_scene(&mut self.packet);

                scene.fixed_update(&mut ctx, &mut view);
            }

            self.context.time.do_tick(tick_start);
        }

        for &index in &self.active_scenes {
            let scene = self.scenes.get_mut(index);
            let (mut ctx, mut view) = self.context.split_scene(&mut self.packet);

            scene.update(&mut ctx, &mut view);
        }

        for &index in &self.active_scenes {
            let scene = self.scenes.get_mut(index);
            let (mut ctx, mut draw) = self.context.split_draw(&mut self.packet);

            scene.draw(&mut ctx, &mut draw);
        }

        let viewport = self.packet.viewport;
        self.context.cameras.write_to(&mut self.packet, viewport);

        // Hand the finished frame to the main thread (latest wins) and wake it.
        let stale = {
            let mut slot = self.frame_slot.lock();

            let mut stale = slot.replace(FrameSubmission {
                packet: self.packet.clone(),
                layers: mem::take(&mut self.context.layers),
            });

            // A replaced frame was never rendered, but imgui was already told
            // its texture requests were serviced. Move those uploads onto the
            // submission that replaced it (ahead of its own) so they still
            // reach the GPU, in order.
            if let (Some(stale), Some(current)) = (stale.as_mut(), slot.as_mut()) {
                if !stale.layers.imgui.textures.is_empty() {
                    let carried = mem::take(&mut stale.layers.imgui.textures);
                    let own = mem::replace(&mut current.layers.imgui.textures, carried);
                    current.layers.imgui.textures.extend(own);
                }
            }

            stale
        };

        // Refill the layers for the next frame with already-allocated buffers:
        // either a stale frame the main thread never rendered, or one it sent
        // back after rendering.
        let mut next = match stale {
            Some(stale) => stale.layers,
            None => self.recycled.try_recv().unwrap_or_default(),
        };

        next.clear();
        self.context.layers = next;

        self.context.window.present();
    }

    fn flush(&mut self) {
        self.drain_scene_commands();

        self.packet.clear();
        self.context.input.flush();
    }

    pub fn run_loop(mut self) {
        let imgui = self.context.imgui.clone();

        self.init();

        loop {
            if self.shutdown.try_recv().is_ok() {
                break;
            }

            {
                // A missing context means the main thread already tore this
                // window down, treat it as a shutdown signal
                let Ok(mut active_imgui) = imgui.try_active(self.context.window.id) else {
                    break;
                };

                let mut io = active_imgui.io_mut();

                self.drain_events(&mut io);
            }

            if self.should_exit {
                break;
            }

            self.context.time.update();
            self.frame();
            self.flush();
            self.context.time.wait_for_next_frame();
        }
    }
}

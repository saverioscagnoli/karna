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

    fn drain_events(&mut self) -> bool {
        if let Ok(()) = self.shutdown.try_recv() {
            return true;
        }

        for event in self.events.try_iter() {
            match event {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::CloseRequested => return true,
                    WindowEvent::Resized(w, h) => {
                        // The SDL GPU swapchain resizes itself; only the
                        // cached size needs updating.
                        let size = math::Size::new(w as u32, h as u32);

                        self.context.window.sync_size(size);
                        debug!("Resized window '{}' to {:?}", self.context.window.id, size);
                    }
                    _ => {}
                },

                Event::KeyDown {
                    timestamp,
                    scancode: Some(c),
                    repeat,
                    ..
                } => {
                    let input = &mut self.context.input;

                    if !repeat {
                        self.pending_input_timestamp.get_or_insert(timestamp);
                        input.pressed_keys.insert(c);
                    }

                    input.held_keys.insert(c);
                }

                Event::KeyUp {
                    timestamp,
                    scancode: Some(c),
                    ..
                } => {
                    self.pending_input_timestamp.get_or_insert(timestamp);

                    let input = &mut self.context.input;

                    input.held_keys.remove(&c);
                    input.released_keys.insert(c);
                }

                _ => {}
            }
        }

        false
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
        let stale = self.frame_slot.lock().replace(FrameSubmission {
            packet: self.packet.clone(),
            layers: mem::take(&mut self.context.layers),
        });

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
        self.init();

        loop {
            if self.drain_events() {
                break;
            }

            self.context.time.update();

            self.frame();
            self.flush();

            self.context.time.wait_for_next_frame();
        }
    }
}

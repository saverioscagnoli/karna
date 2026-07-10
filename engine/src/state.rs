use std::any::Any;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use logging::error;
use renderer::DrawCommand;
use renderer::FramePacket;
use winit::event::WindowEvent;

use crate::Draw;
use crate::Scene;
use crate::context::WindowContext;
use crate::scene::SceneCommand;
use crate::scene::Scenes;
use crate::window::Window;

pub struct WindowState {
    should_exit: bool,
    context: WindowContext,

    scenes: Scenes,
    active_scenes: Vec<String>,

    // Immediate renderer handle
    draw: Draw,
    packet: FramePacket,

    event_rx: Receiver<WindowEvent>,
    packet_tx: Sender<FramePacket>,
}

impl WindowState {
    pub fn new(
        window: Window,
        scenes: Scenes,
        active_scenes: Vec<String>,
        event_rx: Receiver<WindowEvent>,
        packet_tx: Sender<FramePacket>,
    ) -> Self {
        let mut state = Self {
            should_exit: false,
            context: WindowContext::new(window),
            scenes,
            active_scenes: Vec::new(),
            draw: Draw::new(),
            packet: FramePacket::default(),
            event_rx,
            packet_tx,
        };

        for label in active_scenes {
            state.activate_scene(label, None);
        }

        state
    }

    fn register_scene<L>(&mut self, label: L, scene: Box<dyn Scene>)
    where
        L: Into<String>,
    {
        self.scenes.register(label, Box::new(move |_| scene));
    }

    fn activate_scene<L>(&mut self, label: L, user_data: Option<Box<dyn Any>>)
    where
        L: Into<String>,
    {
        let label: String = label.into();

        if self.active_scenes.contains(&label) {
            return;
        }

        self.scenes.build(&label, self.context.as_mut());

        if let Some(scene) = self.scenes.get_mut(&label) {
            scene.load(self.context.as_mut());

            if let Some(user_data) = user_data {
                scene.loaded_with(self.context.as_mut(), user_data);
            }
        }

        self.active_scenes.push(label);
    }

    fn deactivate_scene(&mut self, label: &str) {
        self.active_scenes.retain(|s| s != label);
    }

    fn for_each_active_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Box<dyn Scene>, &mut WindowContext),
    {
        let context = &mut self.context;

        for label in &self.active_scenes {
            if let Some(scene) = self.scenes.get_mut(label) {
                f(scene, context);
            }
        }
    }

    fn for_each_active<F>(&mut self, mut f: F)
    where
        F: FnMut(&Box<dyn Scene>, &WindowContext, &mut Draw),
    {
        let WindowState {
            context,
            draw,
            scenes,
            active_scenes,
            ..
        } = self;

        for label in active_scenes.iter() {
            if let Some(scene) = scenes.get(label) {
                f(scene, context, draw);
            }
        }
    }

    fn drain_events(&mut self) {
        for event in self.event_rx.try_iter() {
            match event {
                WindowEvent::CloseRequested => {
                    self.should_exit = true;
                }

                _ => {}
            }
        }
    }

    fn drain_scene_commands(&mut self) {
        for command in self.context.scenes.drain_collect() {
            match command {
                SceneCommand::Register { label, scene } => self.register_scene(label, scene),
                SceneCommand::Activate { label, user_data } => {
                    self.activate_scene(label, user_data)
                }
                SceneCommand::Deactivate { label } => self.deactivate_scene(&label),
            }
        }
    }

    fn frame(&mut self) {
        while let Some(tick_start) = self.context.time.next_tick() {
            self.for_each_active_mut(|s, ctx| s.fixed_update(ctx.as_mut()));
            self.context.time.do_tick(tick_start);
        }

        self.for_each_active_mut(|s, ctx| s.update(ctx.as_mut()));

        self.for_each_active(|s, ctx, draw| {
            let ctx = ctx.as_ref();
            s.draw(ctx, draw);
        });

        let packet = self.draw.take_packet();

        if let Err(e) = self.packet_tx.try_send(packet) {
            error!("Failed to send frame packet: {}", e);
        }
    }

    fn flush(&mut self) {}

    pub fn start(mut self) {
        while !self.should_exit {
            self.drain_events();
            self.frame();
            self.drain_scene_commands();

            self.context.time.wait_for_next_frame(false);
            self.context.window.request_redraw();

            self.flush();
        }
    }
}

use std::any::Any;
use std::mem;

use assets::AssetServer;
use crossbeam_channel::Receiver;
use gpu::GpuState;
use gpu::WindowSurface;
use logging::debug;
use renderer::FramePacket;
use renderer::Renderer;
use utils::profile;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::PhysicalKey;

use crate::Scene;
use crate::UserEvent;
use crate::context::WindowContext;
use crate::scene::SceneCommand;
use crate::scene::Scenes;
use crate::window::Window;

pub struct WindowState {
    should_exit: bool,
    context: WindowContext,
    renderer: Renderer,
    surface: WindowSurface,
    pending_resize: Option<math::Size<u32>>,

    scenes: Scenes,
    active_scenes: Vec<String>,

    proxy: EventLoopProxy<UserEvent>,
    packet: FramePacket,
    event_rx: Receiver<WindowEvent>,
}

impl WindowState {
    pub fn new(
        window: Window,
        renderer: Renderer,
        surface: WindowSurface,
        scenes: Scenes,
        active_scenes: Vec<String>,
        assets: AssetServer,
        proxy: EventLoopProxy<UserEvent>,
        event_rx: Receiver<WindowEvent>,
    ) -> Self {
        let mut state = Self {
            should_exit: false,
            context: WindowContext::new(window, assets),
            renderer,
            surface,
            pending_resize: None,
            scenes,
            active_scenes: Vec::new(),
            packet: FramePacket::default(),
            proxy,
            event_rx,
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
        self.scenes.register(label, Box::new(move |_, _| scene));
    }

    fn activate_scene<L>(&mut self, label: L, user_data: Option<Box<dyn Any>>)
    where
        L: Into<String>,
    {
        let label: String = label.into();

        if self.active_scenes.contains(&label) {
            return;
        }

        let (ctx, mut s) = self.context.split_mut(&mut self.packet);
        self.scenes.build(&label, ctx, &mut s);

        if let Some(scene) = self.scenes.get_mut(&label) {
            let (ctx, mut s) = self.context.split_mut(&mut self.packet);
            scene.load(ctx, &mut s);

            if let Some(user_data) = user_data {
                let (ctx, mut s) = self.context.split_mut(&mut self.packet);
                scene.loaded_with(ctx, &mut s, user_data);
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
        F: FnMut(&Box<dyn Scene>, &WindowContext),
    {
        let WindowState {
            context,
            scenes,
            active_scenes,
            ..
        } = self;

        for label in active_scenes.iter() {
            if let Some(scene) = scenes.get(label) {
                f(scene, context);
            }
        }
    }

    fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.should_exit = true,

            WindowEvent::Resized(size) => {
                self.pending_resize = Some(size.into());
            }

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => match key_event.physical_key {
                PhysicalKey::Code(c) => {
                    if key_event.state.is_pressed() {
                        if !key_event.repeat {
                            self.context.input.pressed_keys.insert(c);
                        }

                        self.context.input.held_keys.insert(c);
                    } else {
                        self.context.input.held_keys.remove(&c);
                        self.context.input.released_keys.insert(c);
                    }
                }

                _ => {}
            },

            WindowEvent::CursorMoved { position, .. } => {
                self.context
                    .input
                    .mouse_position
                    .set([position.x as f32, position.y as f32]);
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if state.is_pressed() {
                    self.context.input.pressed_mouse_buttons.insert(button);
                    self.context.input.held_mouse_buttons.insert(button);
                } else {
                    self.context.input.held_mouse_buttons.remove(&button);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.context.input.wheel_delta.set([x, y]);
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    self.context
                        .input
                        .wheel_delta
                        .set([pos.x as f32, pos.y as f32]);
                }
            },
            _ => {}
        }
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.handle_event(event);
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
        let mut packet = mem::take(&mut self.packet);

        {
            let _p = profile::scope("fixed_update");

            while let Some(tick_start) = self.context.time.next_tick() {
                self.for_each_active_mut(|scene, ctx| {
                    let (ctx, mut s) = ctx.split_mut(&mut packet);
                    scene.fixed_update(ctx, &mut s);
                });

                self.context.time.do_tick(tick_start);
            }
        }

        {
            let _p = profile::scope("update");

            self.for_each_active_mut(|scene, ctx| {
                let (ctx, mut s) = ctx.split_mut(&mut packet);
                scene.update(ctx, &mut s);
            });
        }

        {
            let _p = profile::scope("draw");

            self.for_each_active(|s, ctx| {
                let (ctx, mut draw) = ctx.split(&mut packet);
                s.draw(ctx, &mut draw);
            });
        }

        self.packet = packet;

        self.packet.viewport = self.context.window.size();
        self.packet.world.camera = self.context.world_camera.data();
        self.packet.ui.camera = self.context.ui_camera.data();
        self.packet.debug.camera = self.context.debug_camera.data();
    }

    fn flush(&mut self) {
        self.packet.clear();
        self.context.input.flush();
        self.drain_scene_commands();
    }

    pub fn start(mut self) {
        while !self.should_exit {
            self.drain_events();

            if self.should_exit {
                break;
            }

            if let Some(size) = self.pending_resize.take() {
                self.surface.resize(GpuState::get(), size);
                self.context.world_camera.update(size);
                self.context.ui_camera.update(size);
                self.context.debug_camera.update(size);
                debug!("Resized window to {:?}", size);
            }

            self.context.time.update();

            self.frame();

            {
                let _p = profile::scope("render");
                self.renderer.present(&mut self.surface, &self.packet);
            }

            {
                let _p = profile::scope("wait");
                self.context.time.wait_for_next_frame();
            }

            self.flush();

            profile::end_frame();
        }
    }
}

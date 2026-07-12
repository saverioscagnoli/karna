use std::any::Any;
use std::mem;

use assets::AssetServer;
use crossbeam_channel::Receiver;
use gpu::GpuState;
use gpu::WindowSurface;
use imgui::SharedImgui;
use logging::debug;
use renderer::FramePacket;
use renderer::Renderer;
use winit::event::DeviceEvent;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::keyboard::PhysicalKey;

use crate::AppEvent;
use crate::Scene;
use crate::context::WindowContext;
use crate::scene::SceneCommand;
use crate::scene::Scenes;
use crate::window::Window;

pub struct WindowState {
    should_exit: bool,
    context: WindowContext,
    renderer: Renderer,
    surface: WindowSurface,
    surface_size: math::Size<u32>,
    pending_resize: Option<math::Size<u32>>,

    scenes: Scenes,
    active_scenes: Vec<String>,

    imgui_font_uv: math::Vector4<f32>,

    packet: FramePacket,
    event_rx: Receiver<AppEvent>,
}

impl WindowState {
    pub fn new(
        window: Window,
        renderer: Renderer,
        surface: WindowSurface,
        scenes: Scenes,
        active_scenes: Vec<String>,
        assets: AssetServer,
        imgui: SharedImgui,
        event_rx: Receiver<AppEvent>,
    ) -> Self {
        let window_id = window.id();
        let viewport = window.size();

        imgui.register_window(window_id, viewport.as_f32());

        let (data, size) = {
            let mut ctx = imgui.active(window_id);
            let fonts = ctx.fonts();
            fonts.tex_id = imgui::TextureId::from(1); // non-zero; unused, one atlas
            let tex = fonts.build_rgba32_texture(); // RGBA8
            (tex.data.to_vec(), math::Size::new(tex.width, tex.height))
        };

        let handle = assets.write_scope(|a| a.load_raw(data, size));
        let font_uv = assets.read().get_image(handle).uv;

        let mut state = Self {
            should_exit: false,
            context: WindowContext::new(window, assets, imgui),
            renderer,
            surface,
            surface_size: viewport,
            pending_resize: None,
            scenes,
            active_scenes: Vec::new(),
            imgui_font_uv: font_uv,
            packet: FramePacket::default(),
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

    fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.should_exit = true,

            WindowEvent::Resized(size) => {
                self.pending_resize = Some(math::Size::new(size.width, size.height));
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
                let pos = [position.x as f32, position.y as f32];

                self.context.input.mouse_position.set(pos);
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if state.is_pressed() {
                    self.context.input.pressed_mouse_buttons.insert(*button);
                    self.context.input.held_mouse_buttons.insert(*button);
                } else {
                    self.context.input.held_mouse_buttons.remove(button);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    let delta = [*x, *y];
                    self.context.input.wheel_delta.set(delta);
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    let delta = [pos.x as f32, pos.y as f32];
                    self.context.input.wheel_delta.set(delta);
                }
            },

            _ => {}
        }
    }

    fn handle_imgui_window_event(&mut self, event: &WindowEvent, io: &mut imgui::Io) {
        match event {
            WindowEvent::Resized(size) => {
                io.display_size = [size.width as f32, size.height as f32];
                io.display_framebuffer_scale = [1.0, 1.0];
            }

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state.is_pressed()
                    && let Some(text) = key_event.text.as_deref()
                {
                    for ch in text.chars() {
                        io.add_input_character(ch);
                    }
                }

                match key_event.physical_key {
                    PhysicalKey::Code(c) => {
                        if let Some(ik) = imgui::winit_keycode_to_imgui(c) {
                            io.add_key_event(ik, key_event.state.is_pressed());
                        }
                    }

                    _ => {}
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let pos = [position.x as f32, position.y as f32];
                io.mouse_pos = pos;
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if let Some(b) = imgui::winit_mousebutton_to_imgui(button) {
                    io.add_mouse_button_event(b, state.is_pressed());
                }
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    let delta = [*x, *y];

                    io.mouse_wheel = delta[1];
                    io.mouse_wheel_h = delta[0];
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    let delta = [pos.x as f32, pos.y as f32];

                    io.mouse_wheel = delta[1];
                    io.mouse_wheel_h = delta[0];
                }
            },

            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let delta = [delta.0 as f32, delta.1 as f32];
                self.context.input.mouse_delta.set(delta);
            }

            _ => {}
        }
    }

    fn handle_imgui_device_event(&mut self, event: &DeviceEvent, io: &mut imgui::Io) {
        match event {
            DeviceEvent::MouseMotion { delta, .. } => {
                let delta = [delta.0 as f32, delta.1 as f32];
                io.mouse_delta = delta;
            }

            _ => {}
        }
    }

    fn drain_events(&mut self, io: &mut imgui::Io) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::Window(e) => {
                    self.handle_window_event(&e);
                    self.handle_imgui_window_event(&e, io);
                }
                AppEvent::Device(e) => {
                    self.handle_device_event(&e);
                    self.handle_imgui_device_event(&e, io);
                }
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

    fn frame(&mut self, imgui: &mut imgui::Ui) {
        let mut packet = mem::take(&mut self.packet);

        while let Some(tick_start) = self.context.time.next_tick() {
            self.for_each_active_mut(|scene, ctx| {
                let (ctx, mut s) = ctx.split_mut(&mut packet);
                scene.fixed_update(ctx, &mut s);
            });

            self.context.time.do_tick(tick_start);
        }

        self.for_each_active_mut(|scene, ctx| {
            let (ctx, mut s) = ctx.split_mut(&mut packet);
            scene.update(ctx, &mut s);
        });

        self.for_each_active_mut(|scene, ctx| {
            let (ctx, mut s) = ctx.split_mut(&mut packet);
            scene.imgui_frame(ctx, &mut s, &mut *imgui);
        });

        self.for_each_active(|s, ctx| {
            let (ctx, mut draw) = ctx.split(&mut packet);
            s.draw(ctx, &mut draw);
        });

        self.packet = packet;

        self.packet.viewport = self.surface_size;
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
        let shared_imgui = self.context.imgui.clone();
        let window_id = self.context.window.id();

        while !self.should_exit {
            let mut imgui = shared_imgui.active(window_id);
            let mut io = imgui.io_mut();

            self.drain_events(&mut io);

            if self.should_exit {
                break;
            }

            if let Some(size) = self.pending_resize.take() {
                self.surface.resize(GpuState::get(), size);
                self.surface_size = size;
                self.context.world_camera.update(size);
                self.context.ui_camera.update(size);
                self.context.debug_camera.update(size);

                debug!("Resized window to {:?}", size);
            }

            self.context.time.update();
            io.delta_time = self.context.time.delta();

            self.frame(imgui.new_frame());

            self.packet.imgui.record(imgui.render(), self.imgui_font_uv);

            drop(imgui);

            self.renderer.present(&mut self.surface, &self.packet);
            self.context.time.wait_for_next_frame();

            self.flush();
        }
    }
}

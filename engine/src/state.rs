use std::any::Any;

use assets::AssetServer;
use crossbeam_channel::Receiver;
use gpu::GpuState;
use gpu::WindowSurface;
use imgui::SharedImgui;
use logging::debug;
use logging::trace;
use logging::warn;
use renderer::Layer;
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
    surface: WindowSurface,
    surface_size: math::Size<u32>,
    pending_resize: Option<math::Size<u32>>,

    scenes: Scenes,
    active: Vec<usize>,

    imgui: SharedImgui,
    imgui_font_uv: math::Vector4<f32>,

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
            let baked = imgui::bake_font_atlas(&mut ctx);
            ctx.fonts().set_texture_id(imgui::TextureId::from(1u64));
            baked
        };

        let handle = assets.write_scope(|a| a.load_raw(data, size));
        let font_uv = assets.read().get_image(handle).uv;

        let mut state = Self {
            should_exit: false,
            context: WindowContext::new(window, assets, renderer),
            surface,
            surface_size: viewport,
            pending_resize: None,
            scenes,
            active: Vec::new(),
            imgui,
            imgui_font_uv: font_uv,
            event_rx,
        };

        for label in active_scenes {
            state.activate_scene(label, None);
        }

        state
    }

    fn register_scene<L: Into<String>>(&mut self, label: L, scene: Box<dyn Scene>) {
        self.scenes.insert_built(label, scene);
    }

    fn activate_scene<L: Into<String>>(&mut self, label: L, user_data: Option<Box<dyn Any>>) {
        let label: String = label.into();

        let Some(i) = self.scenes.index_of(&label) else {
            warn!("activate_scene: nothing registered as {label:?}");
            return;
        };

        if self.active.contains(&i) {
            return;
        }

        {
            let (ctx, mut s) = self.context.split_mut();
            self.scenes.build(i, ctx, &mut s);
        }

        let scene = self.scenes.get_mut(i);

        {
            let (ctx, mut s) = self.context.split_mut();
            scene.load(ctx, &mut s);
        }

        if let Some(user_data) = user_data {
            let (ctx, mut s) = self.context.split_mut();
            scene.loaded_with(ctx, &mut s, user_data);
        }

        self.active.push(i);
    }

    fn deactivate_scene(&mut self, label: &str) {
        let Some(i) = self.scenes.index_of(label) else {
            return;
        };

        self.active.retain(|&a| a != i);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        trace!("Received window event: {:?}", event);

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
                io.set_display_size([size.width as f32, size.height as f32]);
                io.set_display_framebuffer_scale([1.0, 1.0]);
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
                io.add_mouse_pos_event([position.x as f32, position.y as f32]);
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if let Some(b) = imgui::winit_mousebutton_to_imgui(button) {
                    io.add_mouse_button_event(b, state.is_pressed());
                }
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    io.add_mouse_wheel_event([*x, *y]);
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    io.add_mouse_wheel_event([pos.x as f32, pos.y as f32]);
                }
            },

            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: &DeviceEvent) {
        trace!("Received device event: {:?}", event);

        match event {
            DeviceEvent::MouseMotion { delta } => {
                let delta = [delta.0 as f32, delta.1 as f32];
                self.context.input.mouse_delta.set(delta);
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

    fn frame(&mut self, imgui: &imgui::Ui) {
        while let Some(tick_start) = self.context.time.next_tick() {
            for &i in &self.active {
                let scene = self.scenes.get_mut(i);
                let (ctx, mut s) = self.context.split_mut();

                scene.fixed_update(ctx, &mut s);
            }

            self.context.time.do_tick(tick_start);
        }

        for &i in &self.active {
            let scene = self.scenes.get_mut(i);
            let (ctx, mut s) = self.context.split_mut();

            scene.update(ctx, &mut s);
        }

        for &i in &self.active {
            let scene = self.scenes.get_mut(i);
            let (ctx, mut d) = self.context.split(imgui);

            scene.draw(ctx, &mut d);
        }

        self.context.packet.viewport = self.surface_size;
        self.context.packet.world.camera = self.context.cameras[Layer::World].data();
        self.context.packet.ui.camera = self.context.cameras[Layer::Ui].data();
        self.context.packet.debug.camera = self.context.cameras[Layer::Debug].data();
    }

    fn flush(&mut self) {
        self.context.packet.clear();
        self.context.input.flush();
        self.drain_scene_commands();
    }

    /// Runs a single frame: drains queued events, applies resizes, updates
    /// scenes, renders and presents. Returns `false` once the window wants
    /// to close.
    ///
    /// Native drives this from a dedicated per-window thread ([`Self::start`]);
    /// the web drives it from `RedrawRequested`, once per animation frame.
    pub(crate) fn frame_once(&mut self) -> bool {
        let shared_imgui = self.imgui.clone();
        let window_id = self.context.window.id();

        let mut imgui = shared_imgui.active(window_id);
        let mut io = imgui.io_mut();

        self.drain_events(&mut io);

        if self.should_exit {
            return false;
        }

        if let Some(size) = self.pending_resize.take() {
            self.surface.resize(GpuState::get(), size);
            self.surface_size = size;
            self.context.cameras[Layer::World].update(size);
            self.context.cameras[Layer::Ui].update(size);
            self.context.cameras[Layer::Debug].update(size);

            debug!("Resized window to {:?}", size);
        }

        self.context.time.update();
        io.set_delta_time(self.context.time.delta());

        self.frame(imgui.frame());

        self.context
            .packet
            .imgui
            .record(imgui.render(), self.imgui_font_uv);

        drop(imgui);

        self.context
            .renderer
            .present(&mut self.surface, &mut self.context.packet);
        self.context.time.wait_for_next_frame();

        self.flush();

        !self.should_exit
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn start(mut self) {
        while self.frame_once() {}
    }
}

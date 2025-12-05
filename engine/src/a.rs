
//mod context;
//mod scene;
//
//use common::{label, utils::Label};
//use math::Size;
//use renderer::SharedGPU;
//use std::sync::Arc;
//use traccia::info;
//use wgpu::naga::FastHashMap;
//use winit::{
//    application::ApplicationHandler,
//    dpi::PhysicalSize,
//    event::WindowEvent,
//    event_loop::{ControlFlow, EventLoop},
//    keyboard::PhysicalKey,
//    window::WindowAttributes,
//};
//
//pub use crate::{context::Context, scene::Scene};
//pub use winit::{event::MouseButton, keyboard::KeyCode};
//
//struct CustomFormatter;
//
//impl traccia::Formatter for CustomFormatter {
//    fn format(&self, record: &traccia::Record) -> String {
//        format!(
//            "{} {}",
//            record.level.default_coloring().to_lowercase(),
//            record.message
//        )
//    }
//}
//
//fn init_logging() {
//    traccia::init_with_config(traccia::Config {
//        level: if cfg!(debug_assertions) {
//            traccia::LogLevel::Debug
//        } else {
//            traccia::LogLevel::Info
//        },
//        format: Some(Box::new(CustomFormatter)),
//        ..Default::default()
//    });
//}
//
//pub struct App {
//    initial_size: Size<u32>,
//    gpu: Option<Arc<SharedGPU>>,
//    contexts: FastHashMap<winit::window::WindowId, (Context, Label)>,
//    pending_windows: Vec<(Label, WindowAttributes)>,
//    scenes: FastHashMap<Label, Box<dyn Scene>>,
//    current_scene: Option<Label>,
//}
//
//impl Default for App {
//    fn default() -> Self {
//        Self {
//            initial_size: Size::new(800, 600),
//            gpu: None,
//            contexts: FastHashMap::default(),
//            pending_windows: Vec::new(),
//            scenes: FastHashMap::default(),
//            current_scene: None,
//        }
//    }
//}
//
//impl App {
//    pub fn new() -> Self {
//        init_logging();
//        Self::default()
//    }
//
//    pub fn with_scene<L: AsRef<str>>(mut self, label: L, scene: Box<dyn Scene>) -> Self {
//        self.scenes.insert(label!(label.as_ref()), scene);
//        self
//    }
//
//    pub fn with_initial_scene<L: AsRef<str>>(mut self, label: L, scene: Box<dyn Scene>) -> Self {
//        let label = label.as_ref();
//
//        self.scenes.insert(label!(label), scene);
//        self.current_scene = Some(label!(label));
//
//        self.pending_windows.push((
//            label!(label),
//            WindowAttributes::default()
//                .with_inner_size(PhysicalSize::from(self.initial_size))
//                .with_resizable(false),
//        ));
//
//        self
//    }
//
//    pub fn with_window<L: AsRef<str>, S: Into<Size<u32>>>(mut self, label: L, size: S) -> Self {
//        let label = label.as_ref();
//        self.pending_windows.push((
//            label!(label),
//            WindowAttributes::default()
//                .with_inner_size(PhysicalSize::from(size.into()))
//                .with_resizable(false),
//        ));
//        self
//    }
//
//    pub fn add_scene<L: AsRef<str>>(&mut self, label: L, scene: Box<dyn Scene>) {
//        self.scenes.insert(label!(label.as_ref()), scene);
//    }
//
//    pub fn set_scene<L: AsRef<str>>(&mut self, label: L) {
//        let label = label!(label.as_ref());
//
//        if self.scenes.get(&label).is_some() {
//            self.current_scene = Some(label)
//        }
//    }
//
//    /// Access the shared GPU for loading atlas images before windows are created
//    pub fn gpu(&self) -> Option<&Arc<SharedGPU>> {
//        self.gpu.as_ref()
//    }
//
//    /// Mutably access the shared GPU
//    pub fn gpu_mut(&mut self) -> Option<&mut Arc<SharedGPU>> {
//        self.gpu.as_mut()
//    }
//
//    pub fn run(mut self) {
//        let event_loop = EventLoop::with_user_event().build().expect(":(");
//
//        event_loop.set_control_flow(ControlFlow::Poll);
//        event_loop.run_app(&mut self).expect(":(");
//    }
//}
//
//impl ApplicationHandler<Context> for App {
//    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
//        // Initialize shared GPU on first resume if not already done
//        if self.gpu.is_none() {
//            self.gpu = Some(Arc::new(pollster::block_on(SharedGPU::new())));
//
//            let info = self.gpu.as_ref().unwrap().info();
//            info!("backend: {}", info.backend);
//            info!("device: {}", info.name);
//            info!("device type: {:?}", info.device_type);
//            info!("driver: {}", info.driver_info);
//        }
//
//        let gpu = self.gpu.clone().unwrap();
//        let pending_windows = std::mem::take(&mut self.pending_windows);
//
//        for (label, attributes) in pending_windows {
//            let window = event_loop
//                .create_window(attributes)
//                .expect("Failed to create window");
//
//            let window = Arc::new(window);
//            let mut context = Context::new(window.clone(), gpu.clone());
//
//            if let Some(scene) = self.scenes.get_mut(&label) {
//                scene.load(&mut context);
//            }
//
//            self.contexts.insert(window.id(), (context, label));
//            window.request_redraw();
//        }
//    }
//
//    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
//        for (ctx, label) in self.contexts.values_mut() {
//            let Some(scene) = self.scenes.get_mut(label) else {
//                continue;
//            };
//
//            ctx.time.frame_start();
//            ctx.time.update();
//
//            while let Some(update_start) = ctx.time.should_tick() {
//                scene.fixed_update(ctx);
//                ctx.time.do_tick(update_start);
//            }
//
//            scene.update(ctx);
//
//            ctx.window.request_redraw();
//            ctx.input.flush();
//        }
//    }
//
//    fn window_event(
//        &mut self,
//        event_loop: &winit::event_loop::ActiveEventLoop,
//        window_id: winit::window::WindowId,
//        event: winit::event::WindowEvent,
//    ) {
//        let Some((ctx, label)) = self.contexts.get_mut(&window_id) else {
//            return;
//        };
//
//        match event {
//            WindowEvent::CloseRequested => {
//                self.contexts.remove(&window_id);
//                if self.contexts.is_empty() {
//                    event_loop.exit();
//                }
//            }
//
//            WindowEvent::Resized(size) => {
//                ctx.render.resize(size.into());
//            }
//
//            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
//                PhysicalKey::Code(code) => {
//                    if event.state.is_pressed() {
//                        if !event.repeat {
//                            ctx.input.pressed_keys.insert(code);
//                        }
//                        ctx.input.held_keys.insert(code);
//                    } else {
//                        ctx.input.held_keys.remove(&code);
//                    }
//                }
//                PhysicalKey::Unidentified(_) => {}
//            },
//
//            WindowEvent::CursorMoved { position, .. } => {
//                ctx.input.mouse_position.x = position.x as f32;
//                ctx.input.mouse_position.y = position.y as f32;
//            }
//
//            WindowEvent::MouseInput { state, button, .. } => {
//                if state.is_pressed() {
//                    if !ctx.input.pressed_mouse.contains(&button) {
//                        ctx.input.pressed_mouse.insert(button);
//                    }
//                    ctx.input.held_mouse.insert(button);
//                } else {
//                    ctx.input.held_mouse.remove(&button);
//                }
//            }
//
//            WindowEvent::RedrawRequested => {
//                if let Some(scene) = self.scenes.get_mut(label) {
//                    scene.render(ctx);
//                }
//
//                ctx.render.present();
//                ctx.time.frame_end();
//
//                spin_sleep::sleep(ctx.time.until_next_frame());
//            }
//
//            _ => {}
//        }
//    }
//}

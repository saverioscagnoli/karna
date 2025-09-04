mod camera;
mod color;
mod instanced;
mod shapes;
mod util;

use crate::{camera::Camera, instanced::InstancedRenderer};
use math::{Size, Vec3, Vec4};
use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};
use traccia::error;
use wgpu::{Backends, PrimitiveTopology};
use winit::window::Window;

// Re-exports
pub use color::*;
pub use shapes::*;

pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: Rc<wgpu::Device>,
    adapter: wgpu::Adapter,
    queue: Rc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    bind_group_layout: wgpu::BindGroupLayout,
    surface_format: wgpu::TextureFormat,
}

impl GpuState {
    pub fn new(window: Arc<Window>) -> Self {
        let (width, height): (u32, u32) = window.inner_size().into();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = match instance.create_surface(window) {
            Ok(s) => s,
            Err(_) => todo!("handle surface error"),
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to create adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::default(),
            label: Some("device"),
            required_features: wgpu::Features::empty(),
            ..Default::default()
        }))
        .expect("Failed to request a device");

        let device = Rc::new(device);
        let queue = Rc::new(queue);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Bind Group Layout"),
        });

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::default(),
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            adapter,
            queue,
            config,
            bind_group_layout,
            surface_format,
        }
    }
}

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct Vertex {
    pos: Vec3,
    color: Vec4,
}

impl Descriptor for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Renderer {
    state: GpuState,
    camera: Camera,

    pixel_renderer: InstancedRenderer<Pixel, 1>,
    quad_renderer: InstancedRenderer<Rect, 6>,

    clear_color: Color,
}

impl Deref for Renderer {
    type Target = GpuState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Renderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Renderer {
    const VERTEX_CAPACITY: usize = 100_000;

    pub fn _new(window: Arc<Window>) -> Self {
        let (width, height): (u32, u32) = window.inner_size().into();
        let state = GpuState::new(window);
        let surface_format = state.surface_format.clone();

        let device = state.device.clone();
        let queue = state.queue.clone();
        let bind_group_layout = state.bind_group_layout.clone();

        let camera = Camera::new(
            &device,
            queue.clone(),
            &bind_group_layout,
            (width, height).into(),
        );

        let pixel_shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Point Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "../../assets/shaders/pixel.wgsl"
                ))),
            });

        let quad_shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Pixel Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "../../assets/shaders/quad.wgsl"
                ))),
            });

        Self {
            state,
            camera,
            clear_color: Color::Black,
            pixel_renderer: InstancedRenderer::<Pixel, 1>::new(
                device.clone(),
                &pixel_shader,
                &bind_group_layout,
                PrimitiveTopology::PointList,
                surface_format.clone(),
                &[Vertex {
                    pos: Vec3::new(0.0, 0.0, 0.0),
                    color: Color::default().into(),
                }],
                &[0],
                Self::VERTEX_CAPACITY,
            ),
            quad_renderer: InstancedRenderer::<Rect, 6>::new(
                device.clone(),
                &quad_shader,
                &bind_group_layout,
                PrimitiveTopology::TriangleList,
                surface_format.clone(),
                &[
                    Vertex {
                        pos: Vec3::new(0.0, 0.0, 0.0), // Top-left
                        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    },
                    Vertex {
                        pos: Vec3::new(1.0, 0.0, 0.0), // Top-right
                        color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                    },
                    Vertex {
                        pos: Vec3::new(1.0, 1.0, 0.0), // Bottom-right
                        color: Vec4::new(0.0, 0.0, 1.0, 1.0),
                    },
                    Vertex {
                        pos: Vec3::new(0.0, 1.0, 0.0), // Bottom-left
                        color: Vec4::new(1.0, 1.0, 0.0, 1.0),
                    },
                ],
                &[0, 1, 2, 2, 3, 0],
                Self::VERTEX_CAPACITY,
            ),
        }
    }

    pub fn vsync(&self) -> bool {
        self.config.present_mode == wgpu::PresentMode::AutoVsync
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        if vsync {
            self.config.present_mode = wgpu::PresentMode::AutoVsync;
        } else {
            self.config.present_mode = wgpu::PresentMode::AutoNoVsync;
        }

        self.surface.configure(&self.device, &self.config);
    }

    pub fn set_clear_color<C: Into<Color>>(&mut self, color: C) {
        self.clear_color = color.into();
    }

    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn _clear(&mut self) {
        self.pixel_renderer.clear();
        self.quad_renderer.clear();
    }

    pub fn _resize(&mut self, size: Size<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);

        self.camera.update_projection(size);
    }

    pub fn _present(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to acquire next swap chain texture: {:?}", e);
                return;
            }
        };

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command encoder"),
            });

        self.pixel_renderer.update_instances(&mut encoder);
        self.quad_renderer.update_instances(&mut encoder);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                label: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);

            self.pixel_renderer.render(&mut render_pass);
            self.quad_renderer.render(&mut render_pass);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}

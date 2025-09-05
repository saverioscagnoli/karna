use crate::{Descriptor, color::Color, render, util, vertex::Vertex};
use math::{Vec2, Vec3, Vec4};
use wgpu::{
    util::DeviceExt,
    wgt::{DrawIndexedIndirectArgs, DrawIndirectArgs},
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RectInstance {
    pub position: Vec2,
    pub scale: Vec2,
    pub color: Vec4,
}

impl Descriptor for RectInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec2>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct RectRenderer {
    pub instances: Vec<RectInstance>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl RectRenderer {
    pub fn new(
        device: &wgpu::Device,
        render_pipeline_layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> Self {
        let vertices: &[Vertex] = &[
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.0),
                color: Color::WHITE.into(),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.0),
                color: Color::WHITE.into(),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.0),
                color: Color::WHITE.into(),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.0),
                color: Color::WHITE.into(),
            },
        ];

        let indices: &[u16] = &[0, 1, 2, 2, 3, 0];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Vertex Buffer"),
            contents: util::as_u8_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Index Buffer"),
            contents: util::as_u8_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Instance Buffer"),
            size: 1024 * std::mem::size_of::<RectInstance>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rect Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), RectInstance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            instances: Vec::new(),
            vertex_buffer,
            index_buffer,
            instance_buffer,
            render_pipeline,
        }
    }

    pub fn update_instance_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.instance_buffer, 0, util::as_u8_slice(&self.instances));
    }

    pub fn flush(
        &mut self,
        queue: &wgpu::Queue,
        indirect_buffer: &wgpu::Buffer,
        render_pass: &mut wgpu::RenderPass,
    ) {
        self.update_instance_buffer(queue);

        let args = DrawIndexedIndirectArgs {
            index_count: 6,
            instance_count: self.instances.len() as u32,
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        };

        queue.write_buffer(indirect_buffer, 0, util::as_u8_slice(&[args]));

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed_indirect(indirect_buffer, 0);

        self.instances.clear();
    }
}

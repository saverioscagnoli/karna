use std::rc::Rc;

use crate::{Vertex, util};
use traccia::info;

pub struct Batcher {
    device: Rc<wgpu::Device>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pipeline: wgpu::RenderPipeline,
    vertex_capacity: u64,
    index_capacity: u64,
}

impl Batcher {
    pub fn new<L: AsRef<str>>(
        label: L,
        device: Rc<wgpu::Device>,
        shader: &wgpu::ShaderModule,
        bind_group_layout: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        topology: wgpu::PrimitiveTopology,
        vertex_capacity: u64,
        index_capacity: u64,
    ) -> Self {
        let label = label.as_ref();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[],
            zero_initialize_workgroup_memory: false,
        };

        info!(
            "Creating pipeline for {} with topology: {:?}",
            label, topology
        );

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            label: Some(label),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: compilation_options.clone(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options,
            }),
            primitive: wgpu::PrimitiveState {
                topology,
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

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{} Vertex Buffer", label)),
            size: (std::mem::size_of::<Vertex>()) as u64 * vertex_capacity,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{} Index Buffer", label)),
            size: (std::mem::size_of::<u32>()) as u64 * index_capacity,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(vertex_capacity as usize),
            indices: Vec::with_capacity(index_capacity as usize),
            pipeline,
            vertex_capacity,
            index_capacity,
        }
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        if !self.vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, util::as_u8_slice(&self.vertices));
        }

        if !self.indices.is_empty() {
            queue.write_buffer(&self.index_buffer, 0, util::as_u8_slice(&self.indices));
        }
    }

    pub fn check_resize_buffers(&mut self) {
        let needed_vertex_capacity = self.vertices.capacity() as u64;
        let needed_index_capacity = self.indices.capacity() as u64;

        let mut vertex_capacity = self.vertex_capacity;
        let mut index_capacity = self.index_capacity;

        while vertex_capacity < needed_vertex_capacity {
            vertex_capacity *= 2;
        }

        while index_capacity < needed_index_capacity {
            index_capacity *= 2;
        }

        if vertex_capacity > self.vertex_capacity {
            info!(
                "Resizing vertex buffer from {} to {} vertices",
                self.vertex_capacity, vertex_capacity
            );

            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (std::mem::size_of::<Vertex>()) as u64 * vertex_capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.vertex_capacity = vertex_capacity;
        }

        if index_capacity > self.index_capacity {
            info!(
                "Resizing index buffer from {} to {} indices",
                self.index_capacity, index_capacity
            );

            self.index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                size: (std::mem::size_of::<u32>()) as u64 * index_capacity,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            self.index_capacity = index_capacity;
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn flush(&mut self, render_pass: &mut wgpu::RenderPass<'_>) -> u32 {
        if self.vertices.is_empty() || self.indices.is_empty() {
            return 0;
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw_indexed(0..self.index_count(), 0, 0..1);

        1
    }
}

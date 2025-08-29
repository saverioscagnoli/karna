use wgpu::util::DeviceExt;

use crate::{Descriptor, Vertex, util};

pub struct InstancedRenderer<T: Descriptor> {
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    instance_data: Vec<T>,
    instance_capacity: usize,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    staging_buffer: wgpu::Buffer,
    current_staging_offset: usize,
}

impl<T: Descriptor> InstancedRenderer<T> {
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        bind_group_layout: &wgpu::BindGroupLayout,
        topology: wgpu::PrimitiveTopology,
        surface_format: wgpu::TextureFormat,
        vertices: &[Vertex],
        indices: &[u16],
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!(
                "Instanced Pipeline Layout<{}>",
                std::any::type_name::<T>()
            )),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!(
                "Instanced Render Pipeline<{}>",
                std::any::type_name::<T>()
            )),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), T::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
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

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Instance Buffer<{}>", std::any::type_name::<T>())),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Vertex Buffer<{}>", std::any::type_name::<T>())),
            contents: util::as_u8_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer<{}>", std::any::type_name::<T>())),
            contents: util::as_u8_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Staging Buffer<{}>", std::any::type_name::<T>())),
            size: (std::mem::size_of::<T>() * 100_000) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            instance_buffer,
            instance_data: Vec::new(),
            instance_capacity: 100_000,
            vertex_buffer,
            index_buffer,
            staging_buffer,
            current_staging_offset: 0,
        }
    }
}

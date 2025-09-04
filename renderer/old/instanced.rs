use crate::{Descriptor, Vertex, util};
use std::{fmt::Debug, rc::Rc};
use traccia::{error, info};
use wgpu::util::DeviceExt;

pub struct RingBuffer<const SIZE: usize> {
    buffers: [wgpu::Buffer; SIZE],
    staging_buffers: [wgpu::Buffer; SIZE],
    current_buffer: usize,
    count: usize,
}

impl<const SIZE: usize> RingBuffer<SIZE> {
    pub fn new(
        device: &wgpu::Device,
        label_prefix: &str,
        buffer_size: u64,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let buffers = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{} Ring Buffer {}", label_prefix, i)),
                size: buffer_size,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let staging_buffers = std::array::from_fn(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{} Staging Ring Buffer {}", label_prefix, i)),
                size: buffer_size,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false, // Changed: Don't map at creation
            })
        });

        Self {
            buffers,
            staging_buffers,
            current_buffer: 0,
            count: 0,
        }
    }

    pub fn advance(&mut self) {
        self.current_buffer = (self.current_buffer + 1) % SIZE;

        if self.count < SIZE {
            self.count += 1;
        }
    }

    pub fn current_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[self.current_buffer]
    }

    pub fn current_staging_buffer(&self) -> &wgpu::Buffer {
        &self.staging_buffers[self.current_buffer]
    }

    pub fn resize_all(
        &mut self,
        device: &wgpu::Device,
        label_prefix: &str,
        new_size: u64,
        usage: wgpu::BufferUsages,
    ) {
        *self = Self::new(device, label_prefix, new_size, usage);
    }
}

pub struct InstancedRenderer<T: Debug + Descriptor, const I: u32> {
    device: Rc<wgpu::Device>,
    render_pipeline: wgpu::RenderPipeline,

    instance_buffer: RingBuffer<3>,
    instance_data: Vec<T>,
    instance_capacity: usize,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    // Flag for lazy update
    dirty: bool,
}

impl<T: Debug + Descriptor, const I: u32> InstancedRenderer<T, I> {
    pub fn new(
        device: Rc<wgpu::Device>,
        shader: &wgpu::ShaderModule,
        bind_group_layout: &wgpu::BindGroupLayout,
        topology: wgpu::PrimitiveTopology,
        surface_format: wgpu::TextureFormat,
        vertices: &[Vertex],
        indices: &[u16],
        instance_capacity: usize,
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
                module: shader,
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
                cull_mode: None,
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Vertex Buffer<{}>", std::any::type_name::<T>())),
            contents: util::as_u8_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer<{}>", std::any::type_name::<T>())),
            contents: util::as_u8_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create ring buffer for instance data
        let buffer_size = (std::mem::size_of::<T>() * instance_capacity) as wgpu::BufferAddress;
        let instance_buffer = RingBuffer::new(
            &device,
            &format!("Instance<{}>", std::any::type_name::<T>()),
            buffer_size,
            wgpu::BufferUsages::VERTEX,
        );

        Self {
            device,
            render_pipeline,
            instance_buffer,
            instance_data: Vec::with_capacity(instance_capacity),
            instance_capacity,
            vertex_buffer,
            index_buffer,
            dirty: false,
        }
    }

    pub fn clear(&mut self) {
        self.instance_data.clear();
        self.dirty = true;
    }

    pub fn push_data(&mut self, data: T) {
        self.instance_data.push(data);
        self.dirty = true;
    }

    fn check_resize_buffer(&mut self) {
        if self.instance_data.len() > self.instance_capacity {
            let old_capacity = self.instance_capacity;
            self.instance_capacity = (self.instance_capacity * 2).max(self.instance_data.len());

            let buffer_size =
                (std::mem::size_of::<T>() * self.instance_capacity) as wgpu::BufferAddress;

            self.instance_buffer.resize_all(
                &self.device,
                &format!("Instance<{}>", std::any::type_name::<T>()),
                buffer_size,
                wgpu::BufferUsages::VERTEX,
            );

            info!(
                "Resized instance ring buffer from {} to {} capacity",
                old_capacity, self.instance_capacity
            );
        }
    }

    pub fn update_instances(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !self.dirty || self.instance_data.is_empty() {
            return;
        }

        self.instance_buffer.advance();
        self.check_resize_buffer();

        let data_size =
            (std::mem::size_of::<T>() * self.instance_data.len()) as wgpu::BufferAddress;
        let staging_buffer = self.instance_buffer.current_staging_buffer();
        let buffer_slice = staging_buffer.slice(0..data_size);

        buffer_slice.map_async(wgpu::MapMode::Write, |result| {
            if let Err(e) = result {
                error!("Failed to map buffer: {:?}", e);
            }
        });

        if let Err(e) = self.device.poll(wgpu::PollType::Wait) {
            error!("Device poll error: {:?}", e);
        }

        {
            let mut mapped_range = buffer_slice.get_mapped_range_mut();
            let data_bytes = util::as_u8_slice(&self.instance_data);
            mapped_range.copy_from_slice(data_bytes);
        }

        staging_buffer.unmap();

        encoder.copy_buffer_to_buffer(
            staging_buffer,
            0,
            self.instance_buffer.current_buffer(),
            0,
            data_size,
        );

        self.dirty = false;
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        if self.instance_data.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.current_buffer().slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..I, 0, 0..self.instance_data.len() as u32);
    }
}

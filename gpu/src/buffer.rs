use std::marker::PhantomData;
use std::mem;
use std::ops::RangeBounds;

use logging::debug;
use wgpu::util::DeviceExt;

use crate::GpuState;

pub use wgpu::BufferUsages;

#[derive(Debug, Clone)]
pub struct Buffer<T> {
    label: String,
    inner: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    /// The max number of T elements that this buffer can hold
    capacity: usize,
    /// Current number of T elements in the buffer
    len: usize,
    _data: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn builder<'a>(label: &'a str) -> BufferBuilder<'a, T> {
        BufferBuilder::new(label)
    }

    pub fn new_filled<L: AsRef<str>>(label: L, usage: wgpu::BufferUsages, data: &[T]) -> Self {
        let label: &str = label.as_ref();
        let gpu = GpuState::get();
        let capacity = data.len();

        let inner = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                usage,
                contents: utils::as_u8_slice(data),
            });

        Self {
            label: label.to_owned(),
            inner,
            usage,
            capacity,
            len: capacity,
            _data: PhantomData,
        }
    }

    pub fn new_with_capacity<L: AsRef<str>>(
        label: L,
        usage: wgpu::BufferUsages,
        capacity: usize,
    ) -> Self {
        let gpu = GpuState::get();
        let label: &str = label.as_ref();
        let size = (mem::size_of::<T>() * capacity.max(1)) as u64;

        let inner = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            label: label.to_owned(),
            inner,
            usage,
            capacity,
            len: 0,
            _data: PhantomData,
        }
    }

    pub fn wgpu(&self) -> &wgpu::Buffer {
        &self.inner
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Resize the buffer to a new capacity
    pub fn resize(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }

        let gpu = GpuState::get();
        let size = (std::mem::size_of::<T>() * new_capacity.max(1)) as u64;

        self.inner = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            size,
            usage: self.usage,
            mapped_at_creation: false,
        });

        debug!(
            "Resized gpu buffer '{}' from {} to {}",
            self.label, self.capacity, new_capacity
        );

        self.capacity = new_capacity;
        self.len = self.len.min(new_capacity);
    }

    /// Write data to the buffer from a specific index
    pub fn write(&mut self, index: usize, data: &[T]) {
        let required = index + data.len();

        if required > self.capacity {
            self.resize(required.next_power_of_two());
        }

        let gpu = GpuState::get();

        gpu.queue.write_buffer(
            &self.inner,
            (index * mem::size_of::<T>()) as u64,
            utils::as_u8_slice(data),
        );

        self.len = self.len.max(required);
    }

    /// Replace all buffer contents
    pub fn write_all(&mut self, data: &[T]) {
        self.write(0, data);
    }

    /// Get a slice of the buffer
    pub fn slice<'a, S: RangeBounds<u64>>(&'a self, bounds: S) -> wgpu::BufferSlice<'a> {
        self.inner.slice(bounds)
    }

    /// Get the entire buffer as a slice
    pub fn slice_all<'a>(&'a self) -> wgpu::BufferSlice<'a> {
        self.inner.slice(..)
    }

    /// Get the byte size of the buffer
    pub fn byte_size(&self) -> u64 {
        (mem::size_of::<T>() * self.capacity) as u64
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear the buffer (sets len to 0, doesn't deallocate)
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

pub struct BufferBuilder<'a, T> {
    label: &'a str,
    usage: wgpu::BufferUsages,
    contents: Option<&'a [T]>,
    capacity: usize,
}

impl<'a, T> BufferBuilder<'a, T> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            usage: wgpu::BufferUsages::empty(),
            contents: None,
            capacity: 0,
        }
    }

    pub fn vertex(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::VERTEX;
        self
    }

    pub fn index(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::INDEX;
        self
    }

    pub fn uniform(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::UNIFORM;
        self
    }

    pub fn storage(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::STORAGE;
        self
    }

    pub fn writable(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::COPY_DST;
        self
    }

    pub fn data(mut self, data: &'a [T]) -> Self {
        self.contents = Some(data);
        self
    }

    pub fn capacity(mut self, cap: usize) -> Self {
        self.capacity = cap;
        self
    }

    pub fn build(self) -> Buffer<T> {
        match self.contents {
            Some(data) => Buffer::new_filled(self.label, self.usage, data),
            None => Buffer::new_with_capacity(self.label, self.usage, self.capacity),
        }
    }
}

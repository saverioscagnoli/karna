use std::marker::PhantomData;
use std::mem;
use std::ops::RangeBounds;

use wgpu::util::DeviceExt;

use crate::GpuState;

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
    pub fn wgpu(&self) -> &wgpu::Buffer {
        &self.inner
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

        self.capacity = new_capacity;
        self.len = self.len.min(new_capacity);
    }

    /// Write data to the buffer at a specific offset
    pub fn write(&self, offset: u64, data: &[T]) {
        let gpu = GpuState::get();

        gpu.queue
            .write_buffer(&self.inner, offset, utils::as_u8_slice(data));
    }

    /// Write a single element at a specific byte offset
    pub fn write_at(&self, byte_offset: u64, data: &[T]) {
        self.write(byte_offset, data);
    }

    /// Write data starting from element index
    pub fn write_from_index(&self, index: usize, data: &[T]) {
        let byte_offset = (index * mem::size_of::<T>()) as u64;
        self.write(byte_offset, data);
    }

    /// Replace all buffer contents
    pub fn write_all(&mut self, data: &[T]) {
        if data.len() > self.capacity {
            self.resize(data.len());
        }

        self.write(0, data);
        self.len = data.len();
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

use macros::Get;
use std::{marker::PhantomData, ops::RangeBounds};
use wgpu::util::DeviceExt;

#[derive(Debug, Get)]
pub struct GpuBuffer<T> {
    #[get]
    inner: wgpu::Buffer,
    label: Option<String>,

    usage: wgpu::BufferUsages,

    #[get(copied)]
    capacity: usize,

    #[get(copied)]
    len: usize, // Current number of T elements (for dynamic buffers)

    _marker: PhantomData<T>,
}

impl<T> GpuBuffer<T> {
    /// Create a new buffer with initial data
    pub fn new<S: Into<String>>(label: S, usage: wgpu::BufferUsages, data: &[T]) -> Self {
        let label_str = label.into();
        let capacity = data.len();
        let size = (std::mem::size_of::<T>() * capacity) as u64;

        let inner = crate::device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&label_str),
            contents: utils::as_u8_slice(data),
            usage,
        });

        Self {
            inner,
            label: Some(label_str),
            usage,
            capacity,
            len: capacity,
            _marker: PhantomData,
        }
    }

    /// Create an empty buffer with a specific capacity
    pub fn with_capacity<S: Into<String>>(
        label: S,
        usage: wgpu::BufferUsages,
        capacity: usize,
    ) -> Self {
        let label_str = label.into();
        let size = (std::mem::size_of::<T>() * capacity.max(1)) as u64;

        let inner = crate::device().create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label_str),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            inner,
            label: Some(label_str),
            usage,
            capacity,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Write data to the buffer at a specific offset
    pub fn write(&self, offset: u64, data: &[T]) {
        crate::queue().write_buffer(&self.inner, offset, utils::as_u8_slice(data));
    }

    /// Write a single element at a specific byte offset
    pub fn write_at(&self, byte_offset: u64, data: &[T]) {
        self.write(byte_offset, data);
    }

    /// Write data starting from element index
    pub fn write_from_index(&self, index: usize, data: &[T]) {
        let byte_offset = (index * std::mem::size_of::<T>()) as u64;
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

    /// Resize the buffer to a new capacity
    pub fn resize(&mut self, new_capacity: usize) {
        if new_capacity == self.capacity {
            return;
        }

        let size = (std::mem::size_of::<T>() * new_capacity.max(1)) as u64;

        self.inner = crate::device().create_buffer(&wgpu::BufferDescriptor {
            label: self.label.as_deref(),
            size,
            usage: self.usage,
            mapped_at_creation: false,
        });

        self.capacity = new_capacity;
        self.len = self.len.min(new_capacity);
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
        (std::mem::size_of::<T>() * self.capacity) as u64
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

// Builder pattern for more flexibility
impl<T> GpuBuffer<T> {
    pub fn builder() -> GpuBufferBuilder<T> {
        GpuBufferBuilder::new()
    }
}

pub struct GpuBufferBuilder<T> {
    label: Option<String>,
    usage: wgpu::BufferUsages,
    capacity: Option<usize>,
    data: Option<Vec<T>>,
}

impl<T> GpuBufferBuilder<T> {
    pub fn new() -> Self {
        Self {
            label: None,
            usage: wgpu::BufferUsages::empty(),
            capacity: None,
            data: None,
        }
    }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn usage(mut self, usage: wgpu::BufferUsages) -> Self {
        self.usage = usage;
        self
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

    pub fn copy_dst(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::COPY_DST;
        self
    }

    pub fn copy_src(mut self) -> Self {
        self.usage |= wgpu::BufferUsages::COPY_SRC;
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn data(mut self, data: Vec<T>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn build(self) -> GpuBuffer<T> {
        let label = self.label.unwrap_or_else(|| "Unnamed Buffer".to_string());

        if let Some(data) = self.data {
            GpuBuffer::new(label, self.usage, &data)
        } else {
            let capacity = self.capacity.unwrap_or(1);
            GpuBuffer::with_capacity(label, self.usage, capacity)
        }
    }
}

impl<T> Default for GpuBufferBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

use macros::Get;
use std::{marker::PhantomData, ops::RangeBounds};
use wgpu::util::DeviceExt;

#[derive(Debug)]
#[derive(Get)]
pub struct Buffer<T> {
    #[get]
    inner: wgpu::Buffer,

    #[get(copied)]
    size: u64,

    _d: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new<S: AsRef<str>>(label: S, usage: wgpu::BufferUsages, data: &[T]) -> Self {
        let size = std::mem::size_of::<T>() as u64 * data.len() as u64;
        let buffer = crate::device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label.as_ref()),
            contents: utils::as_u8_slice(data),
            usage,
        });

        Self {
            inner: buffer,
            size,
            _d: PhantomData,
        }
    }

    pub fn new_empty<S: AsRef<str>>(label: S, usage: wgpu::BufferUsages) -> Self {
        let size = std::mem::size_of::<T>() as u64;
        let buffer = crate::device().create_buffer(&wgpu::BufferDescriptor {
            label: Some(label.as_ref()),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            inner: buffer,
            size,
            _d: PhantomData,
        }
    }

    pub fn new_with_capacity<S: AsRef<str>>(
        label: S,
        usage: wgpu::BufferUsages,
        capacity: usize,
    ) -> Self {
        let size = std::mem::size_of::<T>() as u64 * capacity as u64;
        let buffer = crate::device().create_buffer(&wgpu::BufferDescriptor {
            label: Some(label.as_ref()),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            inner: buffer,
            size,
            _d: PhantomData,
        }
    }

    #[inline]
    pub fn resize(&mut self, new_size: usize) {
        let old_size = self.size;
        let new_size = (new_size * std::mem::size_of::<T>()) as u64;

        if new_size == old_size {
            return;
        }

        let buffer = crate::device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("resized buffer"),
            size: new_size,
            usage: self.inner.usage(),
            mapped_at_creation: false,
        });

        self.inner = buffer;
        self.size = new_size;
    }

    #[inline]
    pub fn write(&self, data: &[T]) {
        crate::queue().write_buffer(&self.inner, 0, utils::as_u8_slice(data));
    }

    #[inline]
    pub fn write_at(&self, offset: u64, data: &[T]) {
        crate::queue().write_buffer(&self.inner, offset, utils::as_u8_slice(data));
    }

    #[inline]
    pub fn slice<R: RangeBounds<wgpu::BufferAddress>>(&self, range: R) -> wgpu::BufferSlice<'_> {
        self.inner.slice(range)
    }
}

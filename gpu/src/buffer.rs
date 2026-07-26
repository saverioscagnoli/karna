use std::marker::PhantomData;
use std::mem;

use logging::debug;
use sdl3::gpu::BufferRegion;
use sdl3::gpu::TransferBufferLocation;
use sdl3::gpu::TransferBufferUsage;

use crate::Gpu;

pub use sdl3::gpu::BufferBinding;
pub use sdl3::gpu::BufferUsageFlags as BufferUsages;

/// A typed GPU buffer plus the transfer buffer used to stream data into it.
///
/// Uploads are recorded into a copy pass, so writes only happen while the
/// renderer has one open (at the start of a frame, before the render pass).
pub struct Buffer<T> {
    label: String,
    inner: sdl3::gpu::Buffer,
    transfer: sdl3::gpu::TransferBuffer,
    usage: BufferUsages,
    /// The max number of T elements that this buffer can hold
    capacity: usize,
    /// Current number of T elements in the buffer
    len: usize,
    _data: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new<L: AsRef<str>>(gpu: &Gpu, label: L, usage: BufferUsages, capacity: usize) -> Self {
        let label: &str = label.as_ref();
        let capacity = capacity.max(1);
        let bytes = (mem::size_of::<T>() * capacity) as u32;

        let inner = gpu
            .device
            .create_buffer()
            .with_usage(usage)
            .with_size(bytes)
            .build()
            .expect("Failed to create buffer");

        let transfer = gpu
            .device
            .create_transfer_buffer()
            .with_usage(TransferBufferUsage::UPLOAD)
            .with_size(bytes)
            .build()
            .expect("Failed to create transfer buffer");

        Self {
            label: label.to_owned(),
            inner,
            transfer,
            usage,
            capacity,
            len: 0,
            _data: PhantomData,
        }
    }

    pub fn raw(&self) -> &sdl3::gpu::Buffer {
        &self.inner
    }

    pub fn binding(&self) -> BufferBinding {
        BufferBinding::new().with_buffer(&self.inner).with_offset(0)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Resize the buffer to a new capacity
    pub fn resize(&mut self, gpu: &Gpu, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }

        let old = self.capacity;
        *self = Self::new(gpu, &self.label, self.usage, new_capacity);

        debug!(
            "Resized gpu buffer '{}' from {} to {}",
            self.label, old, new_capacity
        );
    }

    /// Replace the buffer contents, growing it if needed. Records the upload
    /// into the given copy pass.
    pub fn write_all(&mut self, gpu: &Gpu, copy_pass: &sdl3::gpu::CopyPass, data: &[T]) {
        if data.is_empty() {
            self.len = 0;
            return;
        }

        if data.len() > self.capacity {
            self.resize(gpu, data.len().next_power_of_two());
        }

        let bytes = utils::as_u8_slice(data);

        let mut map = self.transfer.map::<u8>(&gpu.device, true);
        map.mem_mut()[..bytes.len()].copy_from_slice(bytes);
        map.unmap();

        copy_pass.upload_to_gpu_buffer(
            TransferBufferLocation::new()
                .with_transfer_buffer(&self.transfer)
                .with_offset(0),
            BufferRegion::new()
                .with_buffer(&self.inner)
                .with_offset(0)
                .with_size(bytes.len() as u32),
            true,
        );

        self.len = data.len();
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

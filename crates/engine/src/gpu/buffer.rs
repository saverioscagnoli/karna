use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem;
use std::ops::BitOr;
use std::ptr;
use std::ptr::NonNull;
use std::rc::Rc;

use sdl3::SDL_AcquireGPUCommandBuffer;
use sdl3::SDL_BeginGPUCopyPass;
use sdl3::SDL_CreateGPUBuffer;
use sdl3::SDL_CreateGPUTransferBuffer;
use sdl3::SDL_EndGPUCopyPass;
use sdl3::SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_READ;
use sdl3::SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_WRITE;
use sdl3::SDL_GPU_BUFFERUSAGE_GRAPHICS_STORAGE_READ;
use sdl3::SDL_GPU_BUFFERUSAGE_INDEX;
use sdl3::SDL_GPU_BUFFERUSAGE_INDIRECT;
use sdl3::SDL_GPU_BUFFERUSAGE_VERTEX;
use sdl3::SDL_GPUBuffer;
use sdl3::SDL_GPUBufferCreateInfo;
use sdl3::SDL_GPUBufferRegion;
use sdl3::SDL_GPUBufferUsageFlags;
use sdl3::SDL_GPUDevice;
use sdl3::SDL_GPUTransferBuffer;
use sdl3::SDL_GPUTransferBufferCreateInfo;
use sdl3::SDL_GPUTransferBufferLocation;
use sdl3::SDL_GPUTransferBufferUsage;
use sdl3::SDL_GetError;
use sdl3::SDL_MapGPUTransferBuffer;
use sdl3::SDL_ReleaseGPUBuffer;
use sdl3::SDL_ReleaseGPUTransferBuffer;
use sdl3::SDL_SetGPUBufferName;
use sdl3::SDL_SubmitGPUCommandBuffer;
use sdl3::SDL_UnmapGPUTransferBuffer;
use sdl3::SDL_UploadToGPUBuffer;

use crate::gpu::Gpu;

pub struct Mapped<'a> {
    owner: &'a mut GpuTransferBuffer,
    ptr: NonNull<c_void>,
    cursor: u32,
}

impl Mapped<'_> {
    pub fn write<T>(&mut self, data: &[T]) -> Result<u32, BufferError> {
        let bytes = u32::try_from(mem::size_of_val(data)).map_err(|_| BufferError::TooLarge)?;
        let offset = (self.cursor + 15) & !15;
        let end = offset.checked_add(bytes).ok_or(BufferError::TooLarge)?;

        if end > self.owner.capacity {
            return Err(BufferError::TooLarge);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                data.as_ptr().cast::<u8>(),
                self.ptr.as_ptr().cast::<u8>().add(offset as usize),
                bytes as usize,
            );
        }

        self.cursor = end;
        Ok(offset)
    }

    pub fn write_aligned<T>(&mut self, data: &[T], align: u32) -> Result<u32, BufferError> {
        debug_assert!(align.is_power_of_two());

        let bytes = u32::try_from(mem::size_of_val(data)).map_err(|_| BufferError::TooLarge)?;
        let offset = (self.cursor + align - 1) & !(align - 1);
        let end = offset.checked_add(bytes).ok_or(BufferError::TooLarge)?;

        if end > self.owner.capacity {
            return Err(BufferError::TooLarge);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                data.as_ptr().cast::<u8>(),
                self.ptr.as_ptr().cast::<u8>().add(offset as usize),
                bytes as usize,
            );
        }

        self.cursor = end;
        Ok(offset)
    }
}

impl Drop for Mapped<'_> {
    fn drop(&mut self) {
        unsafe { SDL_UnmapGPUTransferBuffer(self.owner.device, self.owner.raw.as_ptr()) }
    }
}

pub struct GpuTransferBuffer {
    device: *mut SDL_GPUDevice,
    raw: NonNull<SDL_GPUTransferBuffer>,
    capacity: u32,
}

impl GpuTransferBuffer {
    pub fn new(device: *mut SDL_GPUDevice, capacity: u32) -> Self {
        let capacity = capacity.max(1);
        let raw = unsafe {
            let mut info: SDL_GPUTransferBufferCreateInfo = mem::zeroed();

            info.usage = SDL_GPUTransferBufferUsage::SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD;
            info.size = capacity;

            NonNull::new(SDL_CreateGPUTransferBuffer(device, &info))
                .expect("Failed to create transfer buffer")
        };

        Self {
            device,
            raw,
            capacity,
        }
    }

    pub fn raw(&self) -> *mut SDL_GPUTransferBuffer {
        self.raw.as_ptr()
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn reserve(&mut self, size: u32) -> Result<(), BufferError> {
        if size <= self.capacity {
            return Ok(());
        }

        let new_cap = size
            .max(self.capacity.saturating_mul(2))
            .checked_next_power_of_two()
            .ok_or(BufferError::TooLarge)?;

        *self = Self::new(self.device, new_cap);

        Ok(())
    }

    pub fn map(&mut self, cycle: bool) -> Result<Mapped<'_>, BufferError> {
        let ptr = unsafe { SDL_MapGPUTransferBuffer(self.device, self.raw.as_ptr(), cycle) };
        let ptr = NonNull::new(ptr).ok_or_else(sdl_error)?;

        Ok(Mapped {
            owner: self,
            ptr,
            cursor: 0,
        })
    }
}

impl Drop for GpuTransferBuffer {
    fn drop(&mut self) {
        unsafe { SDL_ReleaseGPUTransferBuffer(self.device, self.raw.as_ptr()) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferUsage(SDL_GPUBufferUsageFlags);

impl BufferUsage {
    pub const VERTEX: Self = Self(SDL_GPU_BUFFERUSAGE_VERTEX);
    pub const INDEX: Self = Self(SDL_GPU_BUFFERUSAGE_INDEX);
    pub const INDIRECT: Self = Self(SDL_GPU_BUFFERUSAGE_INDIRECT);
    pub const GRAPHICS_STORAGE_READ: Self = Self(SDL_GPU_BUFFERUSAGE_GRAPHICS_STORAGE_READ);
    pub const COMPUTE_STORAGE_READ: Self = Self(SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_READ);
    pub const COMPUTE_STORAGE_WRITE: Self = Self(SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_WRITE);

    pub const fn bits(self) -> SDL_GPUBufferUsageFlags {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for BufferUsage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug)]
pub enum BufferError {
    TooLarge,
    SDL(String),
}

fn sdl_error() -> BufferError {
    unsafe {
        let p = SDL_GetError();
        BufferError::SDL(std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned())
    }
}

pub struct GpuBuffer<T> {
    label: String,
    gpu: Rc<Gpu>,
    raw: NonNull<SDL_GPUBuffer>,
    usage: BufferUsage,
    capacity: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> GpuBuffer<T> {
    pub fn new<L>(gpu: Rc<Gpu>, label: L, capacity: usize, usage: BufferUsage) -> Self
    where
        L: AsRef<str>,
    {
        const {
            assert!(
                mem::size_of::<T>() > 0,
                "zero-sized T has no GPU representation"
            )
        };

        let label = label.as_ref();
        let capacity = capacity.max(1);
        let size = (mem::size_of::<T>() * capacity) as u32;

        let raw = unsafe {
            let mut info: SDL_GPUBufferCreateInfo = mem::zeroed();

            info.usage = usage.bits();
            info.size = size;

            let buf = NonNull::new(SDL_CreateGPUBuffer(gpu.device, &info))
                .expect("Failed to create buffer");

            if let Ok(c) = std::ffi::CString::new(label) {
                SDL_SetGPUBufferName(gpu.device, buf.as_ptr(), c.as_ptr());
            }

            buf
        };

        Self {
            label: label.to_string(),
            gpu,
            raw,
            usage,
            capacity,
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn raw(&self) -> *mut SDL_GPUBuffer {
        self.raw.as_ptr()
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn byte_len(&self) -> u32 {
        (mem::size_of::<T>() * self.len) as u32
    }

    pub(crate) fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity);
        self.len = len;
    }
}

impl<T> Drop for GpuBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            SDL_ReleaseGPUBuffer(self.gpu.device, self.raw.as_ptr());
        }
    }
}

impl Gpu {
    pub fn upload<T>(&self, dst: &mut GpuBuffer<T>, data: &[T]) -> Result<(), BufferError>
    where
        T: Copy,
    {
        if data.is_empty() {
            return Ok(());
        }

        let bytes = u32::try_from(mem::size_of_val(data)).map_err(|_| BufferError::TooLarge)?;
        let mut staging = self.staging.borrow_mut();

        staging.reserve(bytes)?;

        let src_offset = {
            let mut mapped = staging.map(true)?;
            mapped.write(data)?
        };

        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.device);

            if cmd.is_null() {
                return Err(sdl_error());
            }

            let pass = SDL_BeginGPUCopyPass(cmd);
            let source = SDL_GPUTransferBufferLocation {
                transfer_buffer: staging.raw(),
                offset: src_offset,
            };

            let destination = SDL_GPUBufferRegion {
                buffer: dst.raw(),
                offset: 0,
                size: bytes,
            };

            SDL_UploadToGPUBuffer(pass, &source, &destination, true);
            SDL_EndGPUCopyPass(pass);

            if !SDL_SubmitGPUCommandBuffer(cmd) {
                return Err(sdl_error());
            }

            dst.set_len(data.len());
            Ok(())
        }
    }
}

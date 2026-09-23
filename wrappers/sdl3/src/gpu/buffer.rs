use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem;
use core::ops::BitOr;
use core::ptr;
use core::ptr::NonNull;

use alloc::ffi::CString;
use alloc::string::String;
use alloc::string::ToString;
use sdl3_sys::SDL_AcquireGPUCommandBuffer;
use sdl3_sys::SDL_BeginGPUCopyPass;
use sdl3_sys::SDL_CreateGPUBuffer;
use sdl3_sys::SDL_CreateGPUTransferBuffer;
use sdl3_sys::SDL_EndGPUCopyPass;
use sdl3_sys::SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_READ;
use sdl3_sys::SDL_GPU_BUFFERUSAGE_COMPUTE_STORAGE_WRITE;
use sdl3_sys::SDL_GPU_BUFFERUSAGE_GRAPHICS_STORAGE_READ;
use sdl3_sys::SDL_GPU_BUFFERUSAGE_INDEX;
use sdl3_sys::SDL_GPU_BUFFERUSAGE_INDIRECT;
use sdl3_sys::SDL_GPU_BUFFERUSAGE_VERTEX;
use sdl3_sys::SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD;
use sdl3_sys::SDL_GPUBuffer;
use sdl3_sys::SDL_GPUBufferCreateInfo;
use sdl3_sys::SDL_GPUBufferRegion;
use sdl3_sys::SDL_GPUBufferUsageFlags;
use sdl3_sys::SDL_GPUDevice;
use sdl3_sys::SDL_GPUTransferBuffer;
use sdl3_sys::SDL_GPUTransferBufferCreateInfo;
use sdl3_sys::SDL_GPUTransferBufferLocation;
use sdl3_sys::SDL_MapGPUTransferBuffer;
use sdl3_sys::SDL_ReleaseGPUBuffer;
use sdl3_sys::SDL_ReleaseGPUTransferBuffer;
use sdl3_sys::SDL_SetGPUBufferName;
use sdl3_sys::SDL_SubmitGPUCommandBuffer;
use sdl3_sys::SDL_UnmapGPUTransferBuffer;
use sdl3_sys::SDL_UploadToGPUBuffer;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;

use crate::gpu::Device;

pub struct Mapped<'a> {
    owner: &'a mut GpuTransferBuffer,
    ptr: NonNull<c_void>,
    cursor: u32,
}

impl Mapped<'_> {
    pub fn write<T>(&mut self, data: &[T]) -> Result<u32, SdlError> {
        let bytes =
            u32::try_from(mem::size_of_val(data)).map_err(|_| SdlError::new("Value too large"))?;
        let offset = (self.cursor + 15) & !15;
        let end = offset
            .checked_add(bytes)
            .ok_or(SdlError::new("Value too large"))?;

        if end > self.owner.capacity {
            return Err(SdlError::new("Value too large"));
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

    pub fn write_aligned<T>(&mut self, data: &[T], align: u32) -> Result<u32, SdlError> {
        debug_assert!(align.is_power_of_two());

        let bytes =
            u32::try_from(mem::size_of_val(data)).map_err(|_| SdlError::new("Value too large"))?;
        let offset = (self.cursor + align - 1) & !(align - 1);

        let end = offset
            .checked_add(bytes)
            .ok_or(SdlError::new("Buffer too large"))?;

        if end > self.owner.capacity {
            return Err(SdlError::new("Buffer too large"));
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

            info.usage = SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD;
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

    pub fn reserve(&mut self, size: u32) -> Result<(), SdlError> {
        if size <= self.capacity {
            return Ok(());
        }

        let new_cap = size
            .max(self.capacity.saturating_mul(2))
            .checked_next_power_of_two()
            .ok_or(SdlError::new("Buffer too large"))?;

        *self = Self::new(self.device, new_cap);

        Ok(())
    }

    pub fn map(&mut self, cycle: bool) -> Result<Mapped<'_>, SdlError> {
        let ptr = unsafe { SDL_MapGPUTransferBuffer(self.device, self.raw.as_ptr(), cycle) };
        let ptr = NonNull::new(ptr).ok_or_else(get_error)?;

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

pub struct GpuBuffer<T> {
    label: String,
    device: Device,
    raw: NonNull<SDL_GPUBuffer>,
    usage: BufferUsage,
    capacity: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> GpuBuffer<T> {
    pub fn new<L>(device: Device, label: L, capacity: usize, usage: BufferUsage) -> Self
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

            let buf = NonNull::new(SDL_CreateGPUBuffer(device.as_ptr(), &info))
                .expect("Failed to create buffer");

            if let Ok(c) = CString::new(label) {
                SDL_SetGPUBufferName(device.as_ptr(), buf.as_ptr(), c.as_ptr());
            }

            buf
        };

        Self {
            label: label.to_string(),
            device,
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

    pub fn reserve(&mut self, capacity: usize) {
        if capacity <= self.capacity {
            return;
        }

        let grown = capacity.max(self.capacity.saturating_mul(2));
        let label = self.label.clone();
        let device = self.device.share();
        let usage = self.usage;

        *self = Self::new(device, label, grown.next_power_of_two(), usage);
    }
}

impl<T> Drop for GpuBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            SDL_ReleaseGPUBuffer(self.device.as_ptr(), self.raw.as_ptr());
        }
    }
}

impl Device {
    pub fn upload<T>(&self, dst: &mut GpuBuffer<T>, data: &[T]) -> Result<(), SdlError>
    where
        T: Copy,
    {
        if data.is_empty() {
            return Ok(());
        }

        let bytes =
            u32::try_from(mem::size_of_val(data)).map_err(|_| SdlError::new("Value too large"))?;

        dst.reserve(data.len());

        let mut staging = self.staging().borrow_mut();

        staging.reserve(bytes)?;

        let src_offset = {
            let mut mapped = staging.map(true)?;
            mapped.write(data)?
        };

        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.0.as_ptr());

            if cmd.is_null() {
                return Err(get_error());
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
                return Err(get_error());
            }

            dst.set_len(data.len());
            Ok(())
        }
    }
}

mod buffer;
mod pass;
mod pipeline;
mod shader;
mod texture;

pub use buffer::*;
pub use pass::*;
pub use pipeline::*;
pub use shader::*;
pub use texture::*;

use core::cell::RefCell;
use core::ffi::CStr;
use core::mem::ManuallyDrop;
use core::ptr;

use alloc::ffi::CString;
use alloc::rc::Rc;
use sdl3_sys::SDL_ClaimWindowForGPUDevice;
use sdl3_sys::SDL_CreateGPUDevice;
use sdl3_sys::SDL_CreateWindow;
use sdl3_sys::SDL_DestroyGPUDevice;
use sdl3_sys::SDL_GPU_SHADERFORMAT_DXBC;
use sdl3_sys::SDL_GPU_SHADERFORMAT_DXIL;
use sdl3_sys::SDL_GPU_SHADERFORMAT_MSL;
use sdl3_sys::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3_sys::SDL_GPUDevice;
use sdl3_sys::SDL_GPUShaderFormat;
use sdl3_sys::SDL_GPUTextureFormat;
use sdl3_sys::SDL_GetGPUDeviceDriver;
use sdl3_sys::SDL_GetGPUShaderFormats;
use sdl3_sys::SDL_GetGPUSwapchainTextureFormat;
use sdl3_sys::SDL_ReleaseWindowFromGPUDevice;
use sdl3_sys::SDL_SetGPUAllowedFramesInFlight;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;
use traccia::debug;

use crate::render::Color;
use crate::window::Window;
use crate::window::WindowFlags;

/// Starting size of the shared upload staging buffer. It grows on demand; this
/// is just big enough that the first few uploads do not have to.
const STAGING_CAPACITY: u32 = 64 * 1024;

struct DeviceInner {
    raw: ptr::NonNull<SDL_GPUDevice>,

    /// Scratch buffer every upload writes through. It holds a raw device
    /// pointer rather than a `Device` because it lives inside the device, and
    /// an `Rc` back to its owner would never free.
    staging: ManuallyDrop<RefCell<GpuTransferBuffer>>,
}

impl DeviceInner {
    fn as_ptr(&self) -> *mut SDL_GPUDevice {
        self.raw.as_ptr()
    }
}

impl Drop for DeviceInner {
    fn drop(&mut self) {
        unsafe {
            // The staging buffer belongs to this device, so it has to be
            // released while the device is still alive.
            ManuallyDrop::drop(&mut self.staging);

            SDL_DestroyGPUDevice(self.as_ptr());
        }
    }
}

pub struct Device(Rc<DeviceInner>);

impl Device {
    pub fn init() -> Result<Self, SdlError> {
        const SHADER_FORMATS: SDL_GPUShaderFormat = SDL_GPU_SHADERFORMAT_SPIRV
            | SDL_GPU_SHADERFORMAT_DXBC
            | SDL_GPU_SHADERFORMAT_DXIL
            | SDL_GPU_SHADERFORMAT_MSL;

        let ptr = unsafe { SDL_CreateGPUDevice(SHADER_FORMATS, false, ptr::null()) };
        let raw = ptr::NonNull::new(ptr).ok_or_else(|| get_error())?;

        let staging = GpuTransferBuffer::new(raw.as_ptr(), STAGING_CAPACITY);

        debug!("GPU Device initalized.");

        Ok(Self(
            DeviceInner {
                raw,
                staging: ManuallyDrop::new(RefCell::new(staging)),
            }
            .into(),
        ))
    }

    pub fn share(&self) -> Self {
        Self(Rc::clone(&self.0))
    }

    pub fn as_ptr(&self) -> *mut SDL_GPUDevice {
        self.0.as_ptr()
    }

    pub(crate) fn staging(&self) -> &RefCell<GpuTransferBuffer> {
        &self.0.staging
    }

    /// The backend SDL picked: `vulkan`, `direct3d12`, `metal`.
    pub fn driver(&self) -> &str {
        let ptr = unsafe { SDL_GetGPUDeviceDriver(self.as_ptr()) };

        if ptr.is_null() {
            return "unknown";
        }

        unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("unknown") }
    }

    /// Which shader bytecode formats this device accepts. Check before handing
    /// [`Shader`] a blob.
    pub fn shader_formats(&self) -> ShaderFormat {
        ShaderFormat::from_bits(unsafe { SDL_GetGPUShaderFormats(self.as_ptr()) })
    }

    /// The texture format of the window's swapchain, which is what a pipeline
    /// drawing to that window has to declare as its color target.
    pub fn swapchain_format(&self, window: &Window) -> SDL_GPUTextureFormat {
        unsafe { SDL_GetGPUSwapchainTextureFormat(self.as_ptr(), window.as_ptr()) }
    }

    /// How many frames may be queued before [`Device::begin_frame`] blocks.
    /// Between 1 and 3; lower trades throughput for latency.
    pub fn set_frames_in_flight(&self, frames: u32) -> Result<(), SdlError> {
        if !unsafe { SDL_SetGPUAllowedFramesInFlight(self.as_ptr(), frames) } {
            return Err(get_error());
        }

        Ok(())
    }

    fn claim_window(&self, window: &Window) -> Result<(), SdlError> {
        unsafe {
            if !SDL_ClaimWindowForGPUDevice(self.0.as_ptr(), window.as_ptr()) {
                return Err(get_error());
            }

            Ok(())
        }
    }

    pub(crate) fn release_window(&self, window: &Window) {
        unsafe { SDL_ReleaseWindowFromGPUDevice(self.0.as_ptr(), window.as_ptr()) }
    }

    pub fn create_window<T, S>(
        &self,
        title: T,
        size: S,
        flags: WindowFlags,
    ) -> Result<Window, SdlError>
    where
        T: AsRef<str>,
        S: Into<math::Size<u32>>,
    {
        let title = CString::new(title.as_ref()).map_err(|_| SdlError::new("InteriorNul"))?;

        let size = size.into().cast::<i32>();
        let ptr = unsafe { SDL_CreateWindow(title.as_ptr(), size.w(), size.h(), flags.to_sdl()) };

        let raw = ptr::NonNull::new(ptr).ok_or_else(|| get_error())?;
        let window = Window {
            raw,
            device: self.share(),
        };

        self.claim_window(&window)?;
        Ok(window)
    }

    /// Presents a frame that is nothing but a clear.
    pub fn clear(&self, window: &Window, color: Color) -> Result<(), SdlError> {
        let Some(mut frame) = self.begin_frame(window)? else {
            return Ok(());
        };

        // An empty pass with a clear load op is the cheapest way to clear.
        drop(frame.render_pass(LoadOp::Clear(color))?);

        frame.submit()
    }
}

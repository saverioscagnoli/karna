pub mod buffer;
pub mod present_mode;
pub mod vertex;

use std::cell::RefCell;
use std::ffi::CStr;
use std::ffi::c_char;
use std::mem;
use std::ptr;

use logging::debug;
use logging::fatal;
use sdl3::SDL_AcquireGPUCommandBuffer;
use sdl3::SDL_BeginGPURenderPass;
use sdl3::SDL_CreateGPUDevice;
use sdl3::SDL_DestroyGPUDevice;
use sdl3::SDL_EndGPURenderPass;
use sdl3::SDL_FColor;
use sdl3::SDL_GPU_SHADERFORMAT_DXIL;
use sdl3::SDL_GPU_SHADERFORMAT_MSL;
use sdl3::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3::SDL_GPUColorTargetInfo;
use sdl3::SDL_GPUDevice;
use sdl3::SDL_GPULoadOp;
use sdl3::SDL_GPUStoreOp;
use sdl3::SDL_GPUTexture;
use sdl3::SDL_GetGPUDeviceDriver;
use sdl3::SDL_GetGPUDeviceProperties;
use sdl3::SDL_GetStringProperty;
use sdl3::SDL_PropertiesID;
use sdl3::SDL_SubmitGPUCommandBuffer;
use sdl3::SDL_WaitAndAcquireGPUSwapchainTexture;

use crate::err::SDL_LastError;
use crate::gpu::buffer::GpuTransferBuffer;
use crate::render::color::Color;
use crate::window::platform::PlatformWindow;

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub backend: String,
    pub driver: String,
}

/// Copies a borrowed C string. SDL owns the memory, so this must not escape
/// as a pointer — copy immediately.
unsafe fn owned(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

unsafe fn prop(props: SDL_PropertiesID, key: &CStr) -> Option<String> {
    if props == 0 {
        return None;
    }

    unsafe { owned(SDL_GetStringProperty(props, key.as_ptr(), ptr::null())) }
}

pub struct Gpu {
    device: *mut SDL_GPUDevice,
    staging: RefCell<GpuTransferBuffer>,
    info: GpuInfo,
}

impl Gpu {
    // Do not call before SDL_INIT
    pub fn init() -> Self {
        unsafe {
            let device = SDL_CreateGPUDevice(
                SDL_GPU_SHADERFORMAT_SPIRV | SDL_GPU_SHADERFORMAT_DXIL | SDL_GPU_SHADERFORMAT_MSL,
                true,
                ptr::null(),
            );

            if device.is_null() {
                fatal!("Failed to initalize gpu device: {}", SDL_LastError());
            }

            debug!("GPU Device initialized.");

            let backend = owned(SDL_GetGPUDeviceDriver(device)).unwrap_or(String::from("unknown"));
            let props = SDL_GetGPUDeviceProperties(device);
            let name = prop(props, c"SDL.gpu.device.name").unwrap_or(String::from("unknown"));
            let driver_name = prop(props, c"SDL.gpu.device.driver_name");
            let driver_version = prop(props, c"SDL.gpu.device.driver_version");

            let driver = match (driver_name, driver_version) {
                (Some(n), Some(v)) => format!("{} {}", n, v),
                (Some(n), None) => n,
                (None, Some(v)) => v,
                (None, None) => String::from("unknown"),
            };

            Self {
                device,
                staging: RefCell::new(GpuTransferBuffer::new(device, 1024)),
                info: GpuInfo {
                    name,
                    backend,
                    driver,
                },
            }
        }
    }

    pub fn raw(&self) -> *mut SDL_GPUDevice {
        self.device
    }

    pub fn info(&self) -> &GpuInfo {
        &self.info
    }

    /// Function just to make the window visible while building the engine
    pub fn clear(&self, window: &PlatformWindow, color: Color) {
        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.device);

            if cmd.is_null() {
                fatal!("Failed to acquire command buffer: {}", SDL_LastError());
            }

            let mut swapchain: *mut SDL_GPUTexture = ptr::null_mut();
            let ok = SDL_WaitAndAcquireGPUSwapchainTexture(
                cmd,
                window.raw(),
                &mut swapchain,
                ptr::null_mut(), // width
                ptr::null_mut(), // height
            );

            if !ok {
                fatal!("Failed to acquire swapchain texture: {}", SDL_LastError());
            }

            if swapchain.is_null() {
                SDL_SubmitGPUCommandBuffer(cmd);
                return;
            }

            let (r, g, b, a) = color.tuple();
            let mut target: SDL_GPUColorTargetInfo = mem::zeroed();

            target.texture = swapchain;
            target.clear_color = SDL_FColor { r, g, b, a };
            target.load_op = SDL_GPULoadOp::SDL_GPU_LOADOP_CLEAR;
            target.store_op = SDL_GPUStoreOp::SDL_GPU_STOREOP_STORE;

            let pass = SDL_BeginGPURenderPass(cmd, &target, 1, ptr::null());
            SDL_EndGPURenderPass(pass);

            if !SDL_SubmitGPUCommandBuffer(cmd) {
                fatal!("Failed to submit command buffer: {}", SDL_LastError());
            }
        }
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        unsafe { SDL_DestroyGPUDevice(self.device) };
        debug!("Dropping GPU device.");
    }
}

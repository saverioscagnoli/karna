use core::ptr;

use alloc::ffi::CString;
use alloc::rc::Rc;
use sdl3_sys::SDL_AcquireGPUCommandBuffer;
use sdl3_sys::SDL_BeginGPURenderPass;
use sdl3_sys::SDL_CancelGPUCommandBuffer;
use sdl3_sys::SDL_ClaimWindowForGPUDevice;
use sdl3_sys::SDL_CreateGPUDevice;
use sdl3_sys::SDL_CreateWindow;
use sdl3_sys::SDL_DestroyGPUDevice;
use sdl3_sys::SDL_EndGPURenderPass;
use sdl3_sys::SDL_GPU_LOADOP_CLEAR;
use sdl3_sys::SDL_GPU_SHADERFORMAT_DXIL;
use sdl3_sys::SDL_GPU_SHADERFORMAT_MSL;
use sdl3_sys::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3_sys::SDL_GPU_STOREOP_STORE;
use sdl3_sys::SDL_GPUColorTargetInfo;
use sdl3_sys::SDL_GPUDevice;
use sdl3_sys::SDL_GPUShaderFormat;
use sdl3_sys::SDL_GPUTexture;
use sdl3_sys::SDL_ReleaseWindowFromGPUDevice;
use sdl3_sys::SDL_SubmitGPUCommandBuffer;
use sdl3_sys::SDL_WINDOW_RESIZABLE;
use sdl3_sys::SDL_WaitAndAcquireGPUSwapchainTexture;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;
use traccia::debug;

use crate::render::Color;
use crate::window::Window;

struct DeviceInner {
    raw: ptr::NonNull<SDL_GPUDevice>,
}

impl DeviceInner {
    fn as_ptr(&self) -> *mut SDL_GPUDevice {
        self.raw.as_ptr()
    }
}

impl Drop for DeviceInner {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyGPUDevice(self.as_ptr());
        }
    }
}

pub struct Device(Rc<DeviceInner>);

impl Device {
    pub fn init() -> Result<Self, SdlError> {
        const SHADER_FORMATS: SDL_GPUShaderFormat =
            SDL_GPU_SHADERFORMAT_SPIRV | SDL_GPU_SHADERFORMAT_DXIL | SDL_GPU_SHADERFORMAT_MSL;

        let ptr = unsafe { SDL_CreateGPUDevice(SHADER_FORMATS, false, ptr::null()) };
        let raw = ptr::NonNull::new(ptr).ok_or_else(|| get_error())?;

        debug!("GPU Device initalized.");

        Ok(Self(DeviceInner { raw }.into()))
    }

    pub fn share(&self) -> Self {
        Self(Rc::clone(&self.0))
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

    pub fn create_window<T, S>(&self, title: T, size: S) -> Result<Window, SdlError>
    where
        T: AsRef<str>,
        S: Into<math::Size<u32>>,
    {
        let title = CString::new(title.as_ref()).map_err(|_| SdlError::new("InteriorNul"))?;

        let size = size.into().cast::<i32>();
        let ptr =
            unsafe { SDL_CreateWindow(title.as_ptr(), size.w(), size.h(), SDL_WINDOW_RESIZABLE) };

        let raw = ptr::NonNull::new(ptr).ok_or_else(|| get_error())?;
        let window = Window {
            raw,
            device: self.share(),
        };

        self.claim_window(&window)?;
        Ok(window)
    }

    pub fn clear(&self, window: &Window, color: Color) -> Result<(), SdlError> {
        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.0.as_ptr());
            if cmd.is_null() {
                return Err(get_error());
            }

            let mut texture: *mut SDL_GPUTexture = ptr::null_mut();
            let (mut w, mut h) = (0u32, 0u32);

            if !SDL_WaitAndAcquireGPUSwapchainTexture(
                cmd,
                window.as_ptr(),
                &mut texture,
                &mut w,
                &mut h,
            ) {
                let err = get_error();
                SDL_CancelGPUCommandBuffer(cmd);
                return Err(err);
            }

            if texture.is_null() {
                SDL_CancelGPUCommandBuffer(cmd);
                return Ok(());
            }

            let target = SDL_GPUColorTargetInfo {
                texture,
                clear_color: color.raw(),
                load_op: SDL_GPU_LOADOP_CLEAR,
                store_op: SDL_GPU_STOREOP_STORE,
                ..Default::default()
            };

            let pass = SDL_BeginGPURenderPass(cmd, &target, 1, ptr::null());
            if pass.is_null() {
                let err = get_error();
                SDL_CancelGPUCommandBuffer(cmd);
                return Err(err);
            }

            SDL_EndGPURenderPass(pass);

            if !SDL_SubmitGPUCommandBuffer(cmd) {
                return Err(get_error());
            }

            Ok(())
        }
    }
}

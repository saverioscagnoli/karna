use core::marker::PhantomData;
use core::mem;
use core::ptr;
use core::ptr::NonNull;

use alloc::vec::Vec;
use sdl3_sys::SDL_AcquireGPUCommandBuffer;
use sdl3_sys::SDL_BeginGPURenderPass;
use sdl3_sys::SDL_BindGPUFragmentSamplers;
use sdl3_sys::SDL_BindGPUGraphicsPipeline;
use sdl3_sys::SDL_BindGPUIndexBuffer;
use sdl3_sys::SDL_BindGPUVertexBuffers;
use sdl3_sys::SDL_BindGPUVertexSamplers;
use sdl3_sys::SDL_CancelGPUCommandBuffer;
use sdl3_sys::SDL_DrawGPUIndexedPrimitives;
use sdl3_sys::SDL_DrawGPUPrimitives;
use sdl3_sys::SDL_EndGPURenderPass;
use sdl3_sys::SDL_GPU_INDEXELEMENTSIZE_16BIT;
use sdl3_sys::SDL_GPU_INDEXELEMENTSIZE_32BIT;
use sdl3_sys::SDL_GPU_LOADOP_CLEAR;
use sdl3_sys::SDL_GPU_LOADOP_DONT_CARE;
use sdl3_sys::SDL_GPU_LOADOP_LOAD;
use sdl3_sys::SDL_GPU_STOREOP_STORE;
use sdl3_sys::SDL_GPUBufferBinding;
use sdl3_sys::SDL_GPUColorTargetInfo;
use sdl3_sys::SDL_GPUCommandBuffer;
use sdl3_sys::SDL_GPUIndexElementSize;
use sdl3_sys::SDL_GPULoadOp;
use sdl3_sys::SDL_GPURenderPass;
use sdl3_sys::SDL_GPUTexture;
use sdl3_sys::SDL_GPUTextureSamplerBinding;
use sdl3_sys::SDL_GPUViewport;
use sdl3_sys::SDL_PushGPUFragmentUniformData;
use sdl3_sys::SDL_PushGPUVertexUniformData;
use sdl3_sys::SDL_Rect;
use sdl3_sys::SDL_SetGPUScissor;
use sdl3_sys::SDL_SetGPUViewport;
use sdl3_sys::SDL_SubmitGPUCommandBuffer;
use sdl3_sys::SDL_WaitAndAcquireGPUSwapchainTexture;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;

use crate::gpu::Device;
use crate::gpu::GpuBuffer;
use crate::gpu::GraphicsPipeline;
use crate::gpu::Sampler;
use crate::gpu::Texture;
use crate::render::Color;
use crate::window::Window;

#[derive(Debug, Clone, Copy)]
pub enum LoadOp {
    Load,
    Clear(Color),
    DontCare,
}

impl LoadOp {
    fn parts(self) -> (SDL_GPULoadOp, Color) {
        match self {
            Self::Load => (SDL_GPU_LOADOP_LOAD, Color::BLACK),
            Self::Clear(color) => (SDL_GPU_LOADOP_CLEAR, color),
            Self::DontCare => (SDL_GPU_LOADOP_DONT_CARE, Color::BLACK),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexSize {
    U16,
    U32,
}

impl IndexSize {
    pub const fn raw(self) -> SDL_GPUIndexElementSize {
        match self {
            Self::U16 => SDL_GPU_INDEXELEMENTSIZE_16BIT,
            Self::U32 => SDL_GPU_INDEXELEMENTSIZE_32BIT,
        }
    }
}

fn color_target(texture: *mut SDL_GPUTexture, load: LoadOp) -> SDL_GPUColorTargetInfo {
    let (load_op, clear) = load.parts();

    SDL_GPUColorTargetInfo {
        texture,
        clear_color: clear.raw(),
        load_op,
        store_op: SDL_GPU_STOREOP_STORE,
        ..Default::default()
    }
}

pub struct Frame {
    device: Device,
    cmd: *mut SDL_GPUCommandBuffer,
    swapchain: *mut SDL_GPUTexture,
    size: math::Size<u32>,
}

impl Frame {
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn raw(&self) -> *mut SDL_GPUCommandBuffer {
        self.cmd
    }

    pub fn swapchain(&self) -> *mut SDL_GPUTexture {
        self.swapchain
    }

    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn render_pass(&mut self, load: LoadOp) -> Result<RenderPass<'_>, SdlError> {
        let target = color_target(self.swapchain, load);
        self.begin(&[target])
    }

    pub fn render_pass_to(
        &mut self,
        targets: &[(&Texture, LoadOp)],
    ) -> Result<RenderPass<'_>, SdlError> {
        let targets: Vec<_> = targets
            .iter()
            .map(|(texture, load)| color_target(texture.raw(), *load))
            .collect();

        self.begin(&targets)
    }

    fn begin(&mut self, targets: &[SDL_GPUColorTargetInfo]) -> Result<RenderPass<'_>, SdlError> {
        let ptr = unsafe {
            SDL_BeginGPURenderPass(
                self.cmd,
                targets.as_ptr(),
                targets.len() as u32,
                ptr::null(),
            )
        };

        let raw = NonNull::new(ptr).ok_or_else(get_error)?;

        Ok(RenderPass {
            cmd: self.cmd,
            raw,
            _frame: PhantomData,
        })
    }

    pub fn submit(mut self) -> Result<(), SdlError> {
        let cmd = mem::replace(&mut self.cmd, ptr::null_mut());

        if !unsafe { SDL_SubmitGPUCommandBuffer(cmd) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn cancel(self) {}
}

impl Drop for Frame {
    fn drop(&mut self) {
        if !self.cmd.is_null() {
            unsafe { SDL_CancelGPUCommandBuffer(self.cmd) };
        }
    }
}

pub struct RenderPass<'a> {
    cmd: *mut SDL_GPUCommandBuffer,
    raw: NonNull<SDL_GPURenderPass>,
    _frame: PhantomData<&'a mut ()>,
}

impl RenderPass<'_> {
    pub fn raw(&self) -> *mut SDL_GPURenderPass {
        self.raw.as_ptr()
    }

    pub fn bind_pipeline(&mut self, pipeline: &GraphicsPipeline) {
        unsafe { SDL_BindGPUGraphicsPipeline(self.raw.as_ptr(), pipeline.raw()) }
    }

    pub fn bind_vertex_buffer<T>(&mut self, slot: u32, buffer: &GpuBuffer<T>, offset: u32) {
        let binding = SDL_GPUBufferBinding {
            buffer: buffer.raw(),
            offset,
        };

        unsafe { SDL_BindGPUVertexBuffers(self.raw.as_ptr(), slot, &binding, 1) }
    }

    pub fn bind_vertex_buffers(&mut self, first_slot: u32, bindings: &[SDL_GPUBufferBinding]) {
        unsafe {
            SDL_BindGPUVertexBuffers(
                self.raw.as_ptr(),
                first_slot,
                bindings.as_ptr(),
                bindings.len() as u32,
            )
        }
    }

    pub fn bind_index_buffer<T>(&mut self, buffer: &GpuBuffer<T>, size: IndexSize, offset: u32) {
        let binding = SDL_GPUBufferBinding {
            buffer: buffer.raw(),
            offset,
        };

        unsafe { SDL_BindGPUIndexBuffer(self.raw.as_ptr(), &binding, size.raw()) }
    }

    pub fn bind_vertex_sampler(&mut self, slot: u32, texture: &Texture, sampler: &Sampler) {
        let binding = texture.binding(sampler);

        unsafe { SDL_BindGPUVertexSamplers(self.raw.as_ptr(), slot, &binding, 1) }
    }

    pub fn bind_fragment_sampler(&mut self, slot: u32, texture: &Texture, sampler: &Sampler) {
        let binding = texture.binding(sampler);

        unsafe { SDL_BindGPUFragmentSamplers(self.raw.as_ptr(), slot, &binding, 1) }
    }

    pub fn bind_fragment_samplers(
        &mut self,
        first_slot: u32,
        bindings: &[SDL_GPUTextureSamplerBinding],
    ) {
        unsafe {
            SDL_BindGPUFragmentSamplers(
                self.raw.as_ptr(),
                first_slot,
                bindings.as_ptr(),
                bindings.len() as u32,
            )
        }
    }

    pub fn push_vertex_uniform<T: Copy>(&mut self, slot: u32, value: &T) {
        unsafe {
            SDL_PushGPUVertexUniformData(
                self.cmd,
                slot,
                (value as *const T).cast(),
                mem::size_of::<T>() as u32,
            )
        }
    }

    pub fn push_fragment_uniform<T: Copy>(&mut self, slot: u32, value: &T) {
        unsafe {
            SDL_PushGPUFragmentUniformData(
                self.cmd,
                slot,
                (value as *const T).cast(),
                mem::size_of::<T>() as u32,
            )
        }
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let viewport = SDL_GPUViewport {
            x,
            y,
            w,
            h,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        unsafe { SDL_SetGPUViewport(self.raw.as_ptr(), &viewport) }
    }

    pub fn set_scissor(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let rect = SDL_Rect { x, y, w, h };

        unsafe { SDL_SetGPUScissor(self.raw.as_ptr(), &rect) }
    }

    pub fn draw(&mut self, vertices: u32) {
        self.draw_instanced(vertices, 1, 0, 0);
    }

    pub fn draw_instanced(
        &mut self,
        vertices: u32,
        instances: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            SDL_DrawGPUPrimitives(
                self.raw.as_ptr(),
                vertices,
                instances,
                first_vertex,
                first_instance,
            )
        }
    }

    pub fn draw_indexed(&mut self, indices: u32) {
        self.draw_indexed_instanced(indices, 1, 0, 0, 0);
    }

    pub fn draw_indexed_instanced(
        &mut self,
        indices: u32,
        instances: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            SDL_DrawGPUIndexedPrimitives(
                self.raw.as_ptr(),
                indices,
                instances,
                first_index,
                vertex_offset,
                first_instance,
            )
        }
    }
}

impl Drop for RenderPass<'_> {
    fn drop(&mut self) {
        unsafe { SDL_EndGPURenderPass(self.raw.as_ptr()) }
    }
}

impl Device {
    pub fn begin_frame(&self, window: &Window) -> Result<Option<Frame>, SdlError> {
        let cmd = unsafe { SDL_AcquireGPUCommandBuffer(self.as_ptr()) };

        if cmd.is_null() {
            return Err(get_error());
        }

        let mut swapchain: *mut SDL_GPUTexture = ptr::null_mut();
        let (mut w, mut h) = (0u32, 0u32);

        let acquired = unsafe {
            SDL_WaitAndAcquireGPUSwapchainTexture(
                cmd,
                window.as_ptr(),
                &mut swapchain,
                &mut w,
                &mut h,
            )
        };

        if !acquired {
            let err = get_error();
            unsafe { SDL_CancelGPUCommandBuffer(cmd) };
            return Err(err);
        }

        if swapchain.is_null() {
            unsafe { SDL_CancelGPUCommandBuffer(cmd) };
            return Ok(None);
        }

        Ok(Some(Frame {
            device: self.share(),
            cmd,
            swapchain,
            size: math::size!(w, h),
        }))
    }
}

mod buffers;
mod shaders;
mod texture;

pub use buffers::*;
pub use shaders::*;

pub(crate) use texture::OpenGLTexture;

pub(crate) fn vertex_attrib_pointer(index: u32, size: i32, stride: i32, offset: i32) {
    unsafe {
        gl::VertexAttribPointer(
            index,
            size,
            gl::FLOAT,
            gl::FALSE,
            stride * std::mem::size_of::<f32>() as i32,
            if offset == 0 {
                std::ptr::null()
            } else {
                (offset as usize * std::mem::size_of::<f32>()) as *const _
            },
        );
        gl::EnableVertexAttribArray(index);
    }
}

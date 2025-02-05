//! This crate provides a simple interface to OpenGL.
//! Essentially is a wrapper so I don't have to write all that unsafe blocks.

use std::ops::BitOr;

/// Re-export, so I dont have to add the `gl` crate to the dependencies in `karna-core`.
pub use gl::load_with;

#[repr(u32)]
pub enum Mask {
    ColorBufferBit = gl::COLOR_BUFFER_BIT,
    DepthBufferBit = gl::DEPTH_BUFFER_BIT,
    StencilBufferBit = gl::STENCIL_BUFFER_BIT,
}

impl BitOr for Mask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        unsafe { std::mem::transmute(self as u32 | rhs as u32) }
    }
}

pub fn clear(mask: Mask) {
    unsafe {
        gl::Clear(mask as u32);
    }
}

pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}

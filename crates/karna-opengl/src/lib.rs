//! This crate provides a simple interface to OpenGL.
//! Essentially is a wrapper so I don't have to write all that unsafe blocks.

pub mod buffers;
pub mod shaders;

use buffers::VertexBuffer;
use karna_log::{debug, KarnaError};
use std::ops::BitOr;

/// Re-export, so I dont have to add the `gl` crate to the dependencies in `karna-core`.
pub use gl::load_with;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

#[repr(u32)]
pub enum Mask {
    ColorBufferBit = gl::COLOR_BUFFER_BIT,
    DepthBufferBit = gl::DEPTH_BUFFER_BIT,
    StencilBufferBit = gl::STENCIL_BUFFER_BIT,
}

#[repr(u32)]
pub enum Cap {
    DepthTest = gl::DEPTH_TEST,
    CullFace = gl::CULL_FACE,
    Blend = gl::BLEND,
}

impl BitOr for Mask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        unsafe { std::mem::transmute(self as u32 | rhs as u32) }
    }
}

/// Clears the buffers specified by the mask.
pub fn clear(mask: Mask) {
    unsafe {
        gl::Clear(mask as u32);
    }
}

/// Sets the clear color.
pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}

/// Enables the specified capability.
pub fn enable(cap: Cap) {
    unsafe {
        gl::Enable(cap as u32);
    }
}

/// Disables the specified capability.
pub fn disable(cap: Cap) {
    unsafe {
        gl::Disable(cap as u32);
    }
}

/// Sets the viewport.
pub fn viewport(x: i32, y: i32, width: u32, height: u32) {
    unsafe {
        gl::Viewport(x, y, width as i32, height as i32);
    }
}

#[repr(u32)]
pub enum DrawMode {
    Points = gl::POINTS,
    LineStrip = gl::LINE_STRIP,
    LineLoop = gl::LINE_LOOP,
    Lines = gl::LINES,
    LineStripAdjacency = gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency = gl::LINES_ADJACENCY,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    Triangles = gl::TRIANGLES,
    TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY,
    Patches = gl::PATCHES,
}

#[repr(u32)]
pub enum DataType {
    Byte = gl::BYTE,
    UnsignedByte = gl::UNSIGNED_BYTE,
    Short = gl::SHORT,
    UnsignedShort = gl::UNSIGNED_SHORT,
    Int = gl::INT,
    UnsignedInt = gl::UNSIGNED_INT,
    Float = gl::FLOAT,
}

pub fn draw_arrays(mode: DrawMode, first: i32, count: i32) {
    unsafe {
        gl::DrawArrays(mode as u32, first, count);
    }
}

pub fn draw_elements(
    mode: DrawMode,
    count: usize,
    type_: DataType,
    indices: *const std::ffi::c_void,
) {
    unsafe {
        gl::DrawElements(mode as u32, count as i32, type_ as u32, indices);
    }
}

/// Specifies the format of the vertex attribute.
/// Use to have a single VAO for all the VBOs.
pub fn vertex_attrib_format(
    index: u32,
    size: i32,
    data_type: DataType,
    normalized: bool,
    offset: usize,
) {
    unsafe {
        gl::VertexAttribFormat(
            index,
            size,
            data_type as u32,
            normalized as u8,
            offset as u32,
        );
        gl::EnableVertexAttribArray(index);
    }
}

/// Specifies the binding of the vertex attribute.
pub fn vertex_attrib_binding(index: u32, bindingindex: u32) {
    unsafe {
        gl::VertexAttribBinding(index, bindingindex);
    }
}

/// Differs from vbo.bind() for some reason
pub fn bind_vertex_buffer(i: u32, buffer: &VertexBuffer, stride: usize) {
    unsafe {
        gl::BindVertexBuffer(i, buffer.id, 0, stride as i32);
    }
}

/// Checks for OpenGL errors.
pub fn get_gl_error() -> Option<KarnaError> {
    let err = unsafe { gl::GetError() };

    match err {
        gl::NO_ERROR => None,
        gl::INVALID_ENUM => Some(KarnaError::OpenGL(
            err,
            "Enumeration parameter is not legal for this function.".to_string(),
        )),

        gl::INVALID_VALUE => Some(KarnaError::OpenGL(
            err,
            "Value parameter is not legal for this function.".to_string(),
        )),

        gl::INVALID_OPERATION => Some(KarnaError::OpenGL(
            err,
            "The set of state for the command is not valid for its parameters.".to_string(),
        )),

        gl::STACK_OVERFLOW => Some(KarnaError::OpenGL(
            err,
            "Stack overflow has occurred.".to_string(),
        )),

        gl::STACK_UNDERFLOW => Some(KarnaError::OpenGL(
            err,
            "Stack underflow has occurred.".to_string(),
        )),

        gl::OUT_OF_MEMORY => Some(KarnaError::OpenGL(
            err,
            "There is not enough memory left to execute the command.".to_string(),
        )),

        gl::INVALID_FRAMEBUFFER_OPERATION => Some(KarnaError::OpenGL(
            err,
            "The framebuffer object is not complete.".to_string(),
        )),

        gl::CONTEXT_LOST => Some(KarnaError::OpenGL(
            err,
            "The OpenGL context has been lost, due to a graphics card reset.".to_string(),
        )),

        _ => Some(KarnaError::OpenGL(
            err,
            "An unknown error has occurred.".to_string(),
        )),
    }
}

/// Add a simple debug logging for OpenGL events.
pub fn opengl_debug() {
    extern "system" fn callback(
        source: u32,
        type_: u32,
        _id: u32,
        severity: u32,
        _length: i32,
        message: *const i8,
        _user_param: *mut std::ffi::c_void,
    ) {
        let source = match source {
            gl::DEBUG_SOURCE_API => "API",
            gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
            gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
            gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
            gl::DEBUG_SOURCE_APPLICATION => "Application",
            gl::DEBUG_SOURCE_OTHER => "Other",
            _ => "Unknown",
        };

        let type_ = match type_ {
            gl::DEBUG_TYPE_ERROR => "Error",
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
            gl::DEBUG_TYPE_PORTABILITY => "Portability",
            gl::DEBUG_TYPE_PERFORMANCE => "Performance",
            gl::DEBUG_TYPE_MARKER => "Marker",
            gl::DEBUG_TYPE_PUSH_GROUP => "Push Group",
            gl::DEBUG_TYPE_POP_GROUP => "Pop Group",
            gl::DEBUG_TYPE_OTHER => "Other",
            _ => "Unknown",
        };

        let severity = match severity {
            gl::DEBUG_SEVERITY_HIGH => "High",
            gl::DEBUG_SEVERITY_MEDIUM => "Medium",
            gl::DEBUG_SEVERITY_LOW => "Low",
            gl::DEBUG_SEVERITY_NOTIFICATION => "Notification",
            _ => "Unknown",
        };

        let message = unsafe { std::ffi::CStr::from_ptr(message) };

        debug!(
            "Opengl | Source: {}, Type: {}, Severity: {}, Message: {}",
            source,
            type_,
            severity,
            message.to_str().unwrap()
        );
    }

    unsafe {
        gl::DebugMessageCallback(Some(callback), std::ptr::null());
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
    }
}

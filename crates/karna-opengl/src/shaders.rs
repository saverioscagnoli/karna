use std::{collections::HashMap, ffi::CString, rc::Rc};

use karna_log::error;
use karna_math::{
    matrix::{Mat2, Mat3, Mat4},
    vector::{Vec2, Vec3, Vec4},
};

#[repr(u32)]
pub enum ShaderKind {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
}

pub enum Uniform {
    Float(f32),
    Int(i32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat2(Mat2),
    Mat3(Mat3),
    Mat4(Mat4),
}

pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn new(kind: ShaderKind, source: &str) -> Self {
        let id = unsafe { gl::CreateShader(kind as u32) };

        unsafe {
            let c_str = CString::new(source).unwrap();
            gl::ShaderSource(id, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(id);

            // Check for compilation errors
            let mut success: gl::types::GLint = 1;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len: gl::types::GLint = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);

                let error = CString::from_vec_unchecked(vec![b' '; len as usize]);
                gl::GetShaderInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );

                error!("Failed to compile shader: {}", error.to_str().unwrap());
            }
        }

        Self { id }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct Program {
    id: u32,
    locations: HashMap<Rc<str>, i32>,
}

impl Program {
    pub fn new(vertex: Shader, fragment: Shader) -> Self {
        let id = unsafe { gl::CreateProgram() };

        unsafe {
            gl::AttachShader(id, vertex.id);
            gl::AttachShader(id, fragment.id);
            gl::LinkProgram(id);
            gl::ValidateProgram(id);
        }

        Self {
            id,
            locations: HashMap::new(),
        }
    }

    fn get_location(&mut self, name: &str) -> i32 {
        if let Some(location) = self.locations.get(name) {
            return *location;
        }

        let name_c = CString::new(name).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, name_c.as_ptr()) };
        self.locations.insert(name.into(), location);

        location
    }

    pub fn set_uniform<S: AsRef<str>>(&mut self, name: S, uniform: Uniform) {
        let name = name.as_ref();
        let location = self.get_location(name);

        match uniform {
            Uniform::Float(value) => unsafe {
                gl::Uniform1f(location, value);
            },
            Uniform::Int(value) => unsafe {
                gl::Uniform1i(location, value);
            },
            Uniform::Vec2(value) => unsafe {
                gl::Uniform2fv(location, 1, value.as_ptr());
            },
            Uniform::Vec3(value) => unsafe {
                gl::Uniform3fv(location, 1, value.as_ptr());
            },
            Uniform::Vec4(value) => unsafe {
                gl::Uniform4fv(location, 1, value.as_ptr());
            },
            Uniform::Mat2(value) => unsafe {
                gl::UniformMatrix2fv(location, 1, gl::FALSE, value.as_ptr());
            },
            Uniform::Mat3(value) => unsafe {
                gl::UniformMatrix3fv(location, 1, gl::FALSE, value.as_ptr());
            },
            Uniform::Mat4(value) => unsafe {
                gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
            },
        }
    }

    pub fn enable(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn disable(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

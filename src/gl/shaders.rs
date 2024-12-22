use crate::math::{Mat4, Vec2, Vec3, Vec4};
use std::{ffi::CString, fs, path::Path};

pub enum Uniform {
    Float(f32),
    Int(i32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat2,
    Mat3,
    Mat4(Mat4),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    id: u32,
}

impl Program {
    pub fn new(vertex: Shader, fragment: Shader) -> Self {
        if vertex.kind != ShaderKind::Vertex || fragment.kind != ShaderKind::Fragment {
            panic!("The inputs must be a vertex and a fragment shader");
        }

        unsafe {
            let id = gl::CreateProgram();

            vertex.attach(id);
            fragment.attach(id);
            gl::LinkProgram(id);

            Self { id }
        }
    }

    pub fn r#use(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn reset() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn set_uniform(&self, name: &str, uniform: Uniform) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());

            if location == -1 {
                println!("Uniform {} not found", name);
                return;
            }

            match uniform {
                Uniform::Float(value) => gl::Uniform1f(location, value),
                Uniform::Int(value) => gl::Uniform1i(location, value),
                Uniform::Vec2(value) => gl::Uniform2fv(location, 1, value.as_ptr()),
                Uniform::Vec3(value) => gl::Uniform3fv(location, 1, value.as_ptr()),
                Uniform::Vec4(value) => gl::Uniform4fv(location, 1, value.as_ptr()),
                Uniform::Mat2 => unimplemented!(),
                Uniform::Mat3 => unimplemented!(),
                Uniform::Mat4(value) => {
                    gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderKind {
    Vertex,
    Fragment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shader {
    id: u32,
    kind: ShaderKind,
}

impl Shader {
    pub fn from_str(src: &str, kind: ShaderKind) -> Self {
        unsafe {
            let src = CString::new(src).unwrap();
            let id = match kind {
                ShaderKind::Vertex => gl::CreateShader(gl::VERTEX_SHADER),
                ShaderKind::Fragment => gl::CreateShader(gl::FRAGMENT_SHADER),
            };

            gl::ShaderSource(id, 1, &src.as_ptr(), std::ptr::null());
            gl::CompileShader(id);

            let mut success = 0;

            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);

            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = vec![0; len as usize];
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut _);

                println!(
                    "Failed to compile vertex shader: {}",
                    String::from_utf8(buffer).unwrap()
                );
            }

            Self { id, kind }
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let src = fs::read_to_string(path).unwrap();
        Self::from_str(&src, ShaderKind::Vertex)
    }

    pub fn kind(&self) -> ShaderKind {
        self.kind
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn attach(&self, program: u32) {
        unsafe {
            gl::AttachShader(program, self.id);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

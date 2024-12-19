use gl::types::{GLchar, GLenum, GLint, GLuint};

pub enum Uniform {
    Int(i32),
    Float(f32),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Vec4(f32, f32, f32, f32),
    Mat4([f32; 16]),
}

pub(crate) unsafe fn create_shader_program(vertex_src: &str, fragment_src: &str) -> GLuint {
    let vertex_shader = compile_shader(vertex_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(fragment_src, gl::FRAGMENT_SHADER);

    let program = gl::CreateProgram();
    gl::AttachShader(program, vertex_shader);
    gl::AttachShader(program, fragment_shader);
    gl::LinkProgram(program);

    let mut success = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(511);
        gl::GetProgramInfoLog(
            program,
            511,
            std::ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        panic!(
            "Program linking failed: {}",
            std::str::from_utf8(&info_log).unwrap()
        );
    }

    gl::DeleteShader(vertex_shader);
    gl::DeleteShader(fragment_shader);

    program
}

pub(crate) unsafe fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
    let shader = gl::CreateShader(shader_type);
    let c_str = std::ffi::CString::new(src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
    gl::CompileShader(shader);

    let mut success = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(511);
        gl::GetShaderInfoLog(
            shader,
            511,
            std::ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        panic!(
            "Shader compilation failed: {}",
            std::str::from_utf8(&info_log).unwrap()
        );
    }

    shader
}

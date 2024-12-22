pub(crate) struct OpenGLTexture {
    id: u32,
}

impl OpenGLTexture {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }

        Self { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn parameteri(&self, name: u32, param: u32) {
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, name, param as i32);
        }
    }

    pub fn image_2d(&self, width: u32, height: u32, data: *const u8) {
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data as *const _,
            );
        }
    }

    pub fn sub_image_2d(&self, x: i32, y: i32, width: u32, height: u32, data: *const u8) {
        unsafe {
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                width as i32,
                height as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data as *const _,
            );
        }
    }
}

impl Drop for OpenGLTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

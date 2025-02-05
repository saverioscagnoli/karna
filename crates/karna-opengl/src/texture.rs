use karna_math::size::Size;

pub enum Filtering {
    Nearest = gl::NEAREST as isize,
    Linear = gl::LINEAR as isize,
}

#[derive(Debug, Clone, Copy)]
pub enum Wrap {
    ClampToEdge = gl::CLAMP_TO_EDGE as isize,
    Repeat = gl::REPEAT as isize,
    MirroredRepeat = gl::MIRRORED_REPEAT as isize,
}

pub struct Texture {
    id: u32,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn new(width: u32, height: u32) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }

        Self { id, width, height }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn size(&self) -> Size<u32> {
        Size::new(self.width, self.height)
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        let mut data = vec![0; (self.width * self.height * 4) as usize];

        unsafe {
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut _,
            );
        }

        data
    }

    pub fn set_data(&self, data: &[u8]) {
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
        }
    }

    pub fn set_sub_data(&self, x: i32, y: i32, width: u32, height: u32, data: &[u8]) {
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
                data.as_ptr() as *const _,
            );
        }
    }

    pub fn set_filtering(&self, min: Filtering, mag: Filtering) {
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag as i32);
        }
    }

    pub fn set_wrap(&self, wrap: Wrap) {
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap as i32);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

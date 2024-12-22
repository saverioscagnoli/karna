pub struct Vao {
    id: u32,
}

impl Vao {
    pub fn new() -> Vao {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }

        Vao { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub struct Vbo {
    id: u32,
}

impl Vbo {
    pub fn new() -> Vbo {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }

        Vbo { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn buffer_data<T>(&self, data: &[T], usage: u32) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const _,
                usage,
            );
        }
    }
}

impl Drop for Vbo {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct Ebo {
    id: u32,
}

impl Ebo {
    pub fn new() -> Ebo {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }

        Ebo { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn buffer_data<T>(&self, data: &[T], usage: u32) {
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const _,
                usage,
            );
        }
    }
}

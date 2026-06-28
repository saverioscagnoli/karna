use num::Num;

pub struct Size<T: Num + Copy> {
    pub width: T,
    pub height: T,
}

impl<T: Num + Copy> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn w(&self) -> T {
        self.width
    }

    pub fn h(&self) -> T {
        self.height
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }
}

impl<T: Num + Copy> From<(T, T)> for Size<T> {
    fn from((width, height): (T, T)) -> Self {
        Self { width, height }
    }
}

impl<T: Num + Copy> Into<(T, T)> for Size<T> {
    fn into(self) -> (T, T) {
        (self.width, self.height)
    }
}

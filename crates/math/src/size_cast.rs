use num_traits::Num;

use crate::CastFrom;
use crate::Size;

impl<T: Num + Copy> Size<T> {
    #[inline]
    pub fn map<U, F>(self, mut f: F) -> Size<U>
    where
        U: Num + Copy,
        F: FnMut(T) -> U,
    {
        Size::new(f(self.width), f(self.height))
    }

    #[inline]
    pub fn cast<U>(self) -> Size<U>
    where
        U: Num + Copy + CastFrom<T>,
    {
        self.map(U::cast_from)
    }

    #[inline]
    pub fn try_cast<U>(self) -> Result<Size<U>, U::Error>
    where
        U: Num + Copy + TryFrom<T>,
    {
        Ok(Size::new(
            U::try_from(self.width)?,
            U::try_from(self.height)?,
        ))
    }
}

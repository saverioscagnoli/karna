use num_traits::Num;

use crate::CastFrom;
use crate::Vector2;
use crate::Vector3;
use crate::Vector4;

impl<T: Num + Copy> Vector2<T> {
    #[inline]
    pub fn map<U, F>(self, mut f: F) -> Vector2<U>
    where
        U: Num + Copy,
        F: FnMut(T) -> U,
    {
        Vector2::new(f(self.x), f(self.y))
    }

    #[inline]
    pub fn cast<U>(self) -> Vector2<U>
    where
        U: Num + Copy + CastFrom<T>,
    {
        self.map(U::cast_from)
    }

    #[inline]
    pub fn try_cast<U>(self) -> Result<Vector2<U>, U::Error>
    where
        U: Num + Copy + TryFrom<T>,
    {
        Ok(Vector2::new(U::try_from(self.x)?, U::try_from(self.y)?))
    }
}

impl<T: Num + Copy> Vector3<T> {
    #[inline]
    pub fn map<U, F>(self, mut f: F) -> Vector3<U>
    where
        U: Num + Copy,
        F: FnMut(T) -> U,
    {
        Vector3::new(f(self.x), f(self.y), f(self.z))
    }

    #[inline]
    pub fn cast<U>(self) -> Vector3<U>
    where
        U: Num + Copy + CastFrom<T>,
    {
        self.map(U::cast_from)
    }

    #[inline]
    pub fn try_cast<U>(self) -> Result<Vector3<U>, U::Error>
    where
        U: Num + Copy + TryFrom<T>,
    {
        Ok(Vector3::new(
            U::try_from(self.x)?,
            U::try_from(self.y)?,
            U::try_from(self.z)?,
        ))
    }
}

impl<T: Num + Copy> Vector4<T> {
    #[inline]
    pub fn map<U, F>(self, mut f: F) -> Vector4<U>
    where
        U: Num + Copy,
        F: FnMut(T) -> U,
    {
        Vector4::new(f(self.x), f(self.y), f(self.z), f(self.w))
    }

    #[inline]
    pub fn cast<U>(self) -> Vector4<U>
    where
        U: Num + Copy + CastFrom<T>,
    {
        self.map(U::cast_from)
    }

    #[inline]
    pub fn try_cast<U>(self) -> Result<Vector4<U>, U::Error>
    where
        U: Num + Copy + TryFrom<T>,
    {
        Ok(Vector4::new(
            U::try_from(self.x)?,
            U::try_from(self.y)?,
            U::try_from(self.z)?,
            U::try_from(self.w)?,
        ))
    }
}

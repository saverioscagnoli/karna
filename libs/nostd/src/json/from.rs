use alloc::string::String;
use alloc::vec::Vec;

use crate::json::Error;
use crate::json::ErrorKind;
use crate::json::Value;

pub trait FromJson: Sized {
    fn from_json(value: &Value) -> Result<Self, Error>;
}

fn type_error(expected: &'static str, value: &Value) -> Error {
    Error::convert(ErrorKind::Type {
        expected,
        found: value.kind(),
    })
}

impl FromJson for Value {
    fn from_json(value: &Value) -> Result<Self, Error> {
        Ok(value.clone())
    }
}

impl FromJson for bool {
    fn from_json(value: &Value) -> Result<Self, Error> {
        value.as_bool().ok_or_else(|| type_error("bool", value))
    }
}

impl FromJson for f64 {
    fn from_json(value: &Value) -> Result<Self, Error> {
        value.as_f64().ok_or_else(|| type_error("number", value))
    }
}

impl FromJson for f32 {
    fn from_json(value: &Value) -> Result<Self, Error> {
        f64::from_json(value).map(|n| n as f32)
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {$(
        impl FromJson for $t {
            fn from_json(value: &Value) -> Result<Self, Error> {
                let n = value.as_f64().ok_or_else(|| type_error("integer", value))?;
                let i = n as i128;

                if i as f64 != n || i < <$t>::MIN as i128 || i > <$t>::MAX as i128 {
                    return Err(Error::convert(ErrorKind::OutOfRange));
                }

                Ok(i as $t)
            }
        }
    )*};
}

impl_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl FromJson for String {
    fn from_json(value: &Value) -> Result<Self, Error> {
        value
            .as_str()
            .map(String::from)
            .ok_or_else(|| type_error("string", value))
    }
}

impl<T: FromJson> FromJson for Option<T> {
    fn from_json(value: &Value) -> Result<Self, Error> {
        match value {
            Value::Null => Ok(None),
            v => T::from_json(v).map(Some),
        }
    }
}

impl<T: FromJson> FromJson for Vec<T> {
    fn from_json(value: &Value) -> Result<Self, Error> {
        let array = value.as_array().ok_or_else(|| type_error("array", value))?;

        array
            .iter()
            .enumerate()
            .map(|(i, v)| T::from_json(v).map_err(|e| e.at_index(i)))
            .collect()
    }
}

impl<T: FromJson, const N: usize> FromJson for [T; N] {
    fn from_json(value: &Value) -> Result<Self, Error> {
        let array = value.as_array().ok_or_else(|| type_error("array", value))?;

        if array.len() != N {
            return Err(Error::convert(ErrorKind::Length {
                expected: N,
                found: array.len(),
            }));
        }

        let items = Vec::<T>::from_json(value)?;

        Ok(items
            .try_into()
            .unwrap_or_else(|_| unreachable!("length checked above")))
    }
}

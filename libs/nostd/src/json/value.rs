use core::ops::Index;

use alloc::string::String;
use alloc::vec::Vec;

use crate::json::Error;
use crate::json::ErrorKind;
use crate::json::FromJson;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

static NULL: Value = Value::Null;

impl Value {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, Value)]> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_object()?
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn to<T: FromJson>(&self) -> Result<T, Error> {
        T::from_json(self)
    }

    pub fn field<T: FromJson>(&self, key: &str) -> Result<T, Error> {
        let Some(object) = self.as_object() else {
            return Err(Error::convert(ErrorKind::Type {
                expected: "object",
                found: self.kind(),
            }));
        };

        match object.iter().rev().find(|(k, _)| k == key) {
            Some((_, v)) => T::from_json(v).map_err(|e| e.at_key(key)),
            None => Err(Error::convert(ErrorKind::MissingField(key.into()))),
        }
    }

    pub fn field_or<T: FromJson>(&self, key: &str, default: T) -> Result<T, Error> {
        match self.get(key) {
            None | Some(Value::Null) => Ok(default),
            Some(v) => T::from_json(v).map_err(|e| e.at_key(key)),
        }
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, key: &str) -> &Value {
        self.get(key).unwrap_or(&NULL)
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Value {
        self.as_array().and_then(|a| a.get(index)).unwrap_or(&NULL)
    }
}

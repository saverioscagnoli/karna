use core::fmt;

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UnexpectedEnd,
    UnexpectedChar(char),
    Expected(&'static str),
    InvalidNumber,
    InvalidEscape,
    InvalidUnicode,
    ControlCharacter,
    TrailingCharacters,
    DepthLimit,
    MissingField(String),
    Type {
        expected: &'static str,
        found: &'static str,
    },
    OutOfRange,
    Length {
        expected: usize,
        found: usize,
    },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "unexpected end of input"),
            Self::UnexpectedChar(c) => write!(f, "unexpected character {c:?}"),
            Self::Expected(what) => write!(f, "expected {what}"),
            Self::InvalidNumber => write!(f, "invalid number"),
            Self::InvalidEscape => write!(f, "invalid escape sequence"),
            Self::InvalidUnicode => write!(f, "invalid unicode escape"),
            Self::ControlCharacter => write!(f, "control character in string"),
            Self::TrailingCharacters => write!(f, "trailing characters after value"),
            Self::DepthLimit => write!(f, "nesting too deep"),
            Self::MissingField(name) => write!(f, "missing field '{name}'"),
            Self::Type { expected, found } => write!(f, "expected {expected}, found {found}"),
            Self::OutOfRange => write!(f, "number out of range"),
            Self::Length { expected, found } => {
                write!(f, "expected array of length {expected}, found {found}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Segment {
    Key(String),
    Index(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    offset: Option<usize>,
    path: Vec<Segment>,
}

impl Error {
    pub(crate) fn parse(kind: ErrorKind, offset: usize) -> Self {
        Self {
            kind,
            offset: Some(offset),
            path: Vec::new(),
        }
    }

    pub(crate) fn convert(kind: ErrorKind) -> Self {
        Self {
            kind,
            offset: None,
            path: Vec::new(),
        }
    }

    pub(crate) fn at_key(mut self, key: &str) -> Self {
        self.path.push(Segment::Key(key.into()));
        self
    }

    pub(crate) fn at_index(mut self, index: usize) -> Self {
        self.path.push(Segment::Index(index));
        self
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    pub fn location(&self, src: &str) -> Option<(usize, usize)> {
        let offset = self.offset?.min(src.len());
        let before = src.get(..offset)?;
        let line = before.matches('\n').count() + 1;
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        let column = before[line_start..].chars().count() + 1;

        Some((line, column))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.path.is_empty() {
            for (i, segment) in self.path.iter().rev().enumerate() {
                match segment {
                    Segment::Key(key) if i == 0 => write!(f, "{key}")?,
                    Segment::Key(key) => write!(f, ".{key}")?,
                    Segment::Index(index) => write!(f, "[{index}]")?,
                }
            }

            write!(f, ": ")?;
        }

        write!(f, "{}", self.kind)?;

        if let Some(offset) = self.offset {
            write!(f, " at byte {offset}")?;
        }

        Ok(())
    }
}

impl core::error::Error for Error {}

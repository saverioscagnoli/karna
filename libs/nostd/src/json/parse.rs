use alloc::string::String;
use alloc::vec::Vec;

use crate::json::Error;
use crate::json::ErrorKind;
use crate::json::Value;

const MAX_DEPTH: usize = 128;

pub fn parse(src: &str) -> Result<Value, Error> {
    let mut p = Parser {
        text: src,
        src: src.as_bytes(),
        pos: 0,
        depth: 0,
    };

    p.ws();
    let value = p.value()?;
    p.ws();

    if p.pos != p.src.len() {
        return Err(p.err(ErrorKind::TrailingCharacters));
    }

    Ok(value)
}

struct Parser<'a> {
    text: &'a str,
    src: &'a [u8],
    pos: usize,
    depth: usize,
}

impl Parser<'_> {
    fn value(&mut self) -> Result<Value, Error> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Value::String),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'n') => self.literal("null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.unexpected()),
            None => Err(self.err(ErrorKind::UnexpectedEnd)),
        }
    }

    fn object(&mut self) -> Result<Value, Error> {
        self.enter()?;
        self.pos += 1;

        let mut entries = Vec::new();

        self.ws();

        if self.eat(b'}') {
            self.depth -= 1;
            return Ok(Value::Object(entries));
        }

        loop {
            self.ws();

            if self.peek() != Some(b'"') {
                return Err(self.expected("string key"));
            }

            let key = self.string()?;

            self.ws();

            if !self.eat(b':') {
                return Err(self.expected("':'"));
            }

            self.ws();
            let value = self.value()?;
            entries.push((key, value));
            self.ws();

            if self.eat(b',') {
                continue;
            }

            if self.eat(b'}') {
                break;
            }

            return Err(self.expected("',' or '}'"));
        }

        self.depth -= 1;

        Ok(Value::Object(entries))
    }

    fn array(&mut self) -> Result<Value, Error> {
        self.enter()?;
        self.pos += 1;

        let mut items = Vec::new();

        self.ws();

        if self.eat(b']') {
            self.depth -= 1;
            return Ok(Value::Array(items));
        }

        loop {
            self.ws();
            items.push(self.value()?);
            self.ws();

            if self.eat(b',') {
                continue;
            }

            if self.eat(b']') {
                break;
            }

            return Err(self.expected("',' or ']'"));
        }

        self.depth -= 1;

        Ok(Value::Array(items))
    }

    fn string(&mut self) -> Result<String, Error> {
        self.pos += 1;

        let mut out = String::new();

        loop {
            let start = self.pos;

            while let Some(&b) = self.src.get(self.pos) {
                if b == b'"' || b == b'\\' || b < 0x20 {
                    break;
                }

                self.pos += 1;
            }

            out.push_str(&self.text[start..self.pos]);

            match self.peek() {
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }

                Some(b'\\') => {
                    self.pos += 1;
                    self.escape(&mut out)?;
                }

                Some(_) => return Err(self.err(ErrorKind::ControlCharacter)),
                None => return Err(self.err(ErrorKind::UnexpectedEnd)),
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<(), Error> {
        let c = match self.peek() {
            Some(b'"') => '"',
            Some(b'\\') => '\\',
            Some(b'/') => '/',
            Some(b'b') => '\u{8}',
            Some(b'f') => '\u{c}',
            Some(b'n') => '\n',
            Some(b'r') => '\r',
            Some(b't') => '\t',
            Some(b'u') => {
                self.pos += 1;
                return self.unicode(out);
            }
            Some(_) => return Err(self.err(ErrorKind::InvalidEscape)),
            None => return Err(self.err(ErrorKind::UnexpectedEnd)),
        };

        self.pos += 1;
        out.push(c);

        Ok(())
    }

    fn unicode(&mut self, out: &mut String) -> Result<(), Error> {
        let start = self.pos - 2;
        let invalid = Error::parse(ErrorKind::InvalidUnicode, start);
        let hi = self.hex4()?;

        let code = match hi {
            0xD800..=0xDBFF => {
                if self.src.get(self.pos..self.pos + 2) != Some(b"\\u") {
                    return Err(invalid);
                }

                self.pos += 2;
                let lo = self.hex4()?;

                if !(0xDC00..=0xDFFF).contains(&lo) {
                    return Err(invalid);
                }

                0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
            }

            0xDC00..=0xDFFF => return Err(invalid),
            _ => hi,
        };

        out.push(char::from_u32(code).ok_or(invalid)?);

        Ok(())
    }

    fn hex4(&mut self) -> Result<u32, Error> {
        let mut n = 0;

        for _ in 0..4 {
            let digit = match self.peek() {
                Some(b) => (b as char)
                    .to_digit(16)
                    .ok_or_else(|| self.err(ErrorKind::InvalidUnicode))?,
                None => return Err(self.err(ErrorKind::UnexpectedEnd)),
            };

            n = n * 16 + digit;
            self.pos += 1;
        }

        Ok(n)
    }

    fn number(&mut self) -> Result<Value, Error> {
        let start = self.pos;

        self.eat(b'-');

        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.digits();
            }
            _ => return Err(self.err(ErrorKind::InvalidNumber)),
        }

        if self.eat(b'.') && !self.digits() {
            return Err(self.err(ErrorKind::InvalidNumber));
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;

            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }

            if !self.digits() {
                return Err(self.err(ErrorKind::InvalidNumber));
            }
        }

        self.text[start..self.pos]
            .parse()
            .map(Value::Number)
            .map_err(|_| Error::parse(ErrorKind::InvalidNumber, start))
    }

    fn digits(&mut self) -> bool {
        let start = self.pos;

        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }

        self.pos > start
    }

    fn literal(&mut self, word: &'static str, value: Value) -> Result<Value, Error> {
        if !self.src[self.pos..].starts_with(word.as_bytes()) {
            return Err(self.expected(word));
        }

        self.pos += word.len();

        Ok(value)
    }

    fn enter(&mut self) -> Result<(), Error> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            return Err(self.err(ErrorKind::DepthLimit));
        }

        Ok(())
    }

    #[inline]
    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    #[inline]
    fn eat(&mut self, b: u8) -> bool {
        if self.peek() == Some(b) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn err(&self, kind: ErrorKind) -> Error {
        Error::parse(kind, self.pos)
    }

    fn expected(&self, what: &'static str) -> Error {
        match self.peek() {
            None => self.err(ErrorKind::UnexpectedEnd),
            Some(_) => self.err(ErrorKind::Expected(what)),
        }
    }

    fn unexpected(&self) -> Error {
        match self.text[self.pos..].chars().next() {
            Some(c) => self.err(ErrorKind::UnexpectedChar(c)),
            None => self.err(ErrorKind::UnexpectedEnd),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::json::FromJson;

    #[test]
    fn scalars() {
        assert_eq!(parse("null"), Ok(Value::Null));
        assert_eq!(parse(" true "), Ok(Value::Bool(true)));
        assert_eq!(parse("false"), Ok(Value::Bool(false)));
        assert_eq!(parse("-12.5e2"), Ok(Value::Number(-1250.0)));
        assert_eq!(parse("0"), Ok(Value::Number(0.0)));
        assert_eq!(parse(r#""hi""#), Ok(Value::String("hi".into())));
    }

    #[test]
    fn nested() {
        let v = parse(r#"{"a": [1, {"b": null}], "c": "d"}"#).unwrap();

        assert_eq!(v["a"][0], Value::Number(1.0));
        assert!(v["a"][1]["b"].is_null());
        assert_eq!(v["c"].as_str(), Some("d"));
        assert!(v["missing"]["deeper"].is_null());
    }

    #[test]
    fn escapes() {
        let v = parse(r#""a\"b\\c\/\n\t\u00e9\ud83d\ude00""#).unwrap();
        assert_eq!(v.as_str(), Some("a\"b\\c/\n\té😀"));

        let v = parse("\"ünïcödé\"").unwrap();
        assert_eq!(v.as_str(), Some("ünïcödé"));
    }

    #[test]
    fn duplicate_keys_last_wins() {
        let v = parse(r#"{"a": 1, "a": 2}"#).unwrap();
        assert_eq!(v["a"], Value::Number(2.0));
    }

    #[test]
    fn rejects_invalid() {
        for src in [
            "",
            "{",
            "[1,]",
            "{\"a\":1,}",
            "01",
            "1.",
            ".5",
            "-",
            "1e",
            "+1",
            "tru",
            "nul",
            "'a'",
            "\"a",
            "\"\\x\"",
            "\"\\ud800\"",
            "\"\\udc00\"",
            "\"a\nb\"",
            "{a:1}",
            "[1 2]",
            "1 2",
            "{\"a\" 1}",
            "NaN",
            "\u{feff}1",
        ] {
            assert!(parse(src).is_err(), "should reject {src:?}");
        }
    }

    #[test]
    fn depth_limit() {
        let deep = "[".repeat(MAX_DEPTH + 1) + &"]".repeat(MAX_DEPTH + 1);
        assert_eq!(parse(&deep).unwrap_err().kind(), &ErrorKind::DepthLimit);

        let ok = "[".repeat(MAX_DEPTH) + &"]".repeat(MAX_DEPTH);
        assert!(parse(&ok).is_ok());
    }

    #[test]
    fn error_location() {
        let src = "{\n  \"a\": 1\n  \"b\": 2\n}";
        let err = parse(src).unwrap_err();

        assert_eq!(err.kind(), &ErrorKind::Expected("',' or '}'"));
        assert_eq!(err.location(src), Some((3, 3)));
    }

    #[test]
    fn conversion() {
        let v =
            parse(r#"{"frame": [0, 0, 32, 32], "fps": 12, "loop": true, "name": "run"}"#).unwrap();

        assert_eq!(v.field::<[u32; 4]>("frame"), Ok([0, 0, 32, 32]));
        assert_eq!(v.field::<f32>("fps"), Ok(12.0));
        assert_eq!(v.field::<bool>("loop"), Ok(true));
        assert_eq!(v.field::<String>("name"), Ok("run".into()));
        assert_eq!(v.field_or::<u32>("speed", 1), Ok(1));
        assert!(v.field::<Option<u32>>("frame").is_err());
    }

    #[test]
    fn conversion_errors() {
        let v = parse(r#"{"frames": [{"w": 1}, {"w": -1}]}"#).unwrap();

        #[derive(Debug)]
        struct Frame;

        impl FromJson for Frame {
            fn from_json(value: &Value) -> Result<Self, Error> {
                value.field::<u32>("w").map(|_| Frame)
            }
        }

        let err = v.field::<Vec<Frame>>("frames").unwrap_err();
        assert_eq!(err.to_string(), "frames[1].w: number out of range");

        let err = v.field::<u32>("nope").unwrap_err();
        assert_eq!(err.to_string(), "missing field 'nope'");

        assert_eq!(u8::from_json(&Value::Number(255.0)), Ok(255));
        assert!(u8::from_json(&Value::Number(256.0)).is_err());
        assert!(u32::from_json(&Value::Number(1.5)).is_err());
        assert_eq!(
            vec![1u8],
            Vec::<u8>::from_json(&parse("[1]").unwrap()).unwrap()
        );
    }
}

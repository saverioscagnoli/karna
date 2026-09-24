//! version  u32 (1)
//! count    u32
//! count times:
//!     path_len  u32
//!     path      path_len bytes, UTF-8, '/'-separated, normalized
//!     data_len  u64
//!     data      data_len bytes

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

const MAGIC: &[u8; 4] = b"KPAK";
const VERSION: u32 = 1;

pub struct Archive {
    files: BTreeMap<String, &'static [u8]>,
}

impl Archive {
    pub fn parse(bytes: &'static [u8]) -> Result<Self, String> {
        let mut r = Reader { bytes, pos: 0 };

        if r.take(4)? != MAGIC {
            return Err("not a karna archive".to_string());
        }

        let version = r.u32()?;

        if version != VERSION {
            return Err(alloc::format!("unsupported archive version {version}"));
        }

        let count = r.u32()?;
        let mut files = BTreeMap::new();

        for _ in 0..count {
            let len = r.u32()? as usize;
            let path = core::str::from_utf8(r.take(len)?)
                .map_err(|_| "archive path is not UTF-8".to_string())?;

            let len = usize::try_from(r.u64()?).map_err(|_| "archive entry too large")?;
            let data = r.take(len)?;

            files.insert(normalize(path), data);
        }

        Ok(Self { files })
    }

    pub fn get(&self, path: &str) -> Option<&'static [u8]> {
        self.files.get(&normalize(path)).copied()
    }

    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

pub fn write<'a>(files: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> Vec<u8> {
    let files = files.into_iter().collect::<Vec<_>>();
    let mut out = Vec::new();

    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(files.len() as u32).to_le_bytes());

    for (path, data) in files {
        let path = normalize(path);

        out.extend_from_slice(&(path.len() as u32).to_le_bytes());
        out.extend_from_slice(path.as_bytes());
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(data);
    }

    out
}

pub fn normalize(path: &str) -> String {
    let mut parts = Vec::new();

    for part in path.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }

    parts.join("/")
}

struct Reader {
    bytes: &'static [u8],
    pos: usize,
}

impl Reader {
    fn take(&mut self, n: usize) -> Result<&'static [u8], String> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&end| end <= self.bytes.len())
            .ok_or_else(|| "archive is truncated".to_string())?;

        let out = &self.bytes[self.pos..end];
        self.pos = end;

        Ok(out)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use super::*;

    #[test]
    fn round_trips() {
        let bytes = write([
            ("main.js", b"export default {}".as_slice()),
            ("./assets\\pcb.png", &[0, 1, 2, 255]),
            ("empty", &[]),
        ]);
        let archive = Archive::parse(Box::leak(bytes.into_boxed_slice())).unwrap();

        assert_eq!(archive.len(), 3);
        assert_eq!(
            archive.get("main.js"),
            Some(b"export default {}".as_slice())
        );
        assert_eq!(
            archive.get("assets/pcb.png"),
            Some([0, 1, 2, 255].as_slice())
        );
        assert_eq!(
            archive.get("./scripts/../assets/pcb.png").map(<[u8]>::len),
            Some(4)
        );
        assert_eq!(archive.get("empty"), Some([].as_slice()));
        assert_eq!(archive.get("missing"), None);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Archive::parse(b"nope").is_err());
        assert!(Archive::parse(b"KPAK\x01\0\0\0\x05\0\0\0").is_err());

        let mut bytes = write([("a", b"abc".as_slice())]);
        bytes.pop();
        assert!(Archive::parse(Box::leak(bytes.into_boxed_slice())).is_err());
    }

    #[test]
    fn normalizes_paths() {
        assert_eq!(normalize("a/./b/../c"), "a/c");
        assert_eq!(normalize("./a\\c"), "a/c");
        assert_eq!(normalize("/a//c/"), "a/c");
        assert_eq!(normalize("../../a"), "a");
    }
}

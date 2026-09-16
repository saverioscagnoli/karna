use alloc::borrow::ToOwned;
use alloc::string::String;

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::ffi;
use core::fmt;
use core::hash::Hash;
use core::hash::Hasher;
use core::ops::Deref;
use core::ops::Range;

pub const SEP: char = '/';

const DRIVE_LETTERS: bool = cfg!(windows);

const INLINE_C_STR: usize = 512;

#[inline]
const fn is_sep(b: u8) -> bool {
    b == b'/' || b == b'\\'
}

#[repr(transparent)]
pub struct Path(str);

impl Path {
    #[inline]
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        // Sound: `repr(transparent)` gives `Path` the layout of `str`, and
        // both references carry the same length metadata.
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Length of the root prefix: `1` for `/foo`, `3` for `C:\foo`, `2` for a
    /// drive-relative `C:foo`, `0` for anything relative.
    fn root_len(&self) -> usize {
        let b = self.as_bytes();

        if DRIVE_LETTERS && b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
            return if b.len() >= 3 && is_sep(b[2]) { 3 } else { 2 };
        }

        if !b.is_empty() && is_sep(b[0]) { 1 } else { 0 }
    }

    pub fn is_absolute(&self) -> bool {
        let b = self.as_bytes();

        if DRIVE_LETTERS && self.root_len() == 3 {
            return true;
        }

        !b.is_empty() && is_sep(b[0])
    }

    #[inline]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    fn name_range(&self) -> Option<Range<usize>> {
        let b = self.as_bytes();
        let root = self.root_len();

        let mut end = b.len();
        while end > root && is_sep(b[end - 1]) {
            end -= 1;
        }

        if end == root {
            return None;
        }

        let mut start = end;
        while start > root && !is_sep(b[start - 1]) {
            start -= 1;
        }

        Some(start..end)
    }

    /// The final component, or `None` for a root, an empty path, or a path
    /// ending in `.` or `..`.
    pub fn file_name(&self) -> Option<&str> {
        let name = &self.0[self.name_range()?];

        match name {
            "." | ".." => None,
            _ => Some(name),
        }
    }

    /// The final component with its extension stripped.
    pub fn file_stem(&self) -> Option<&str> {
        let name = self.file_name()?;

        match name.rfind('.') {
            // A leading dot names a hidden file, it does not start an
            // extension: `.gitignore` is all stem.
            Some(0) | None => Some(name),
            Some(i) => Some(&name[..i]),
        }
    }

    /// The extension of the final component, without the dot.
    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name()?;

        match name.rfind('.') {
            Some(0) | None => None,
            Some(i) => Some(&name[i + 1..]),
        }
    }

    /// The path without its final component, or `None` if there is nothing
    /// left to strip.
    pub fn parent(&self) -> Option<&Path> {
        let b = self.as_bytes();
        let root = self.root_len();
        let name = self.name_range()?;

        if name.start == root {
            return Some(Path::new(&self.0[..root]));
        }

        // Drop the separator before the name, plus any it was doubled with.
        let mut cut = name.start - 1;
        while cut > root && is_sep(b[cut - 1]) {
            cut -= 1;
        }

        Some(Path::new(&self.0[..cut]))
    }

    /// The components of the path, separators and `.` segments dropped.
    ///
    /// Note that this says nothing about whether the path is rooted; ask
    /// [`Path::is_absolute`] for that.
    #[inline]
    pub fn components(&self) -> Components<'_> {
        Components { rest: &self.0 }
    }

    /// Whether `prefix` names a leading run of this path's components.
    pub fn starts_with(&self, prefix: impl AsRef<Path>) -> bool {
        self.strip_prefix(prefix).is_some()
    }

    /// The path relative to `prefix`, or `None` if it is not a prefix.
    ///
    /// Matching is per component, so `a//b` is a prefix of `a/b/c`.
    pub fn strip_prefix(&self, prefix: impl AsRef<Path>) -> Option<&Path> {
        let prefix = prefix.as_ref();

        if !prefix.is_empty() && prefix.is_absolute() != self.is_absolute() {
            return None;
        }

        let mut rest = self.components();

        for want in prefix.components() {
            if rest.next() != Some(want) {
                return None;
            }
        }

        Some(rest.as_path())
    }

    /// This path with `path` appended. An absolute `path` replaces it outright.
    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.push(path);
        buf
    }

    /// This path with its extension replaced. An empty `ext` removes it.
    pub fn with_extension(&self, ext: &str) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.set_extension(ext);
        buf
    }

    /// This path with its final component replaced.
    pub fn with_file_name(&self, name: &str) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.set_file_name(name);
        buf
    }

    #[inline]
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self)
    }

    /// Calls `f` with this path as a NUL terminated C string.
    ///
    /// Short paths are copied onto the stack; only an unusually long one
    /// allocates. The pointer is valid for the duration of the call and no
    /// longer. Anything past an interior NUL is dropped, since C could not see
    /// it anyway.
    ///
    /// A [`PathBuf`] already carries its terminator, so it shadows this with a
    /// version that hands the pointer straight over.
    pub fn with_c_str<R>(&self, f: impl FnOnce(*const ffi::c_char) -> R) -> R {
        let s = self.as_str();
        let s = match s.find('\0') {
            Some(i) => &s[..i],
            None => s,
        };

        if s.len() < INLINE_C_STR {
            let mut buf = [0u8; INLINE_C_STR];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            f(buf.as_ptr().cast())
        } else {
            let mut owned = String::with_capacity(s.len() + 1);
            owned.push_str(s);
            owned.push('\0');
            f(owned.as_ptr().cast())
        }
    }
}

/// An owned, mutable path.
///
/// The backing string always ends in a NUL that is not part of the path, which
/// is what makes [`PathBuf::as_ptr`] free.
#[derive(Clone)]
pub struct PathBuf {
    inner: String,
}

impl PathBuf {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: String::from("\0"),
        }
    }

    /// An empty path with room for `capacity` bytes of path.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut inner = String::with_capacity(capacity + 1);
        inner.push('\0');
        Self { inner }
    }

    #[inline]
    pub fn as_path(&self) -> &Path {
        Path::new(self.as_str())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // The invariant: one trailing NUL, always there, never part of the
        // path. It is one byte, so this is a char boundary.
        &self.inner[..self.inner.len() - 1]
    }

    /// The path as a NUL terminated C string, no copy.
    ///
    /// Valid as long as the `PathBuf` is not mutated.
    #[inline]
    pub fn as_ptr(&self) -> *const ffi::c_char {
        self.inner.as_ptr().cast()
    }

    /// Hands the C string straight to `f`, shadowing [`Path::with_c_str`] so
    /// an owned path never pays for the copy.
    #[inline]
    pub fn with_c_str<R>(&self, f: impl FnOnce(*const ffi::c_char) -> R) -> R {
        f(self.as_ptr())
    }

    /// Appends `path`, separating it from what is already there.
    ///
    /// An absolute `path` replaces the whole thing, matching `std`.
    pub fn push(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();

        if path.is_absolute() {
            self.inner.clear();
            self.push_raw(path.as_str());
            self.inner.push('\0');
            return;
        }

        if path.is_empty() {
            return;
        }

        self.inner.pop();

        let b = self.inner.as_bytes();
        if !b.is_empty() && !is_sep(b[b.len() - 1]) {
            self.inner.push(SEP);
        }

        self.push_raw(path.as_str());
        self.inner.push('\0');
    }

    /// Drops the final component, returning whether there was one to drop.
    pub fn pop(&mut self) -> bool {
        let Some(parent) = self.as_path().parent() else {
            return false;
        };

        // `parent` is a prefix of the same string, so its length is a valid
        // truncation point.
        let len = parent.len();
        self.inner.truncate(len);
        self.inner.push('\0');

        true
    }

    /// Replaces the extension of the final component, or adds one. An empty
    /// `ext` removes it. Returns `false` if there is no final component.
    pub fn set_extension(&mut self, ext: &str) -> bool {
        let Some(name) = self.as_path().name_range() else {
            return false;
        };

        let stem = match self.inner[name.clone()].rfind('.') {
            Some(0) | None => name.len(),
            Some(i) => i,
        };

        self.inner.truncate(name.start + stem);

        if !ext.is_empty() {
            self.inner.push('.');
            self.push_raw(ext);
        }

        self.inner.push('\0');

        true
    }

    /// Replaces the final component, or appends `name` if there is none.
    pub fn set_file_name(&mut self, name: &str) {
        if self.as_path().name_range().is_some() {
            self.pop();
        }

        self.push(name);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.push('\0');
    }

    /// Bytes of path the buffer can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity().saturating_sub(1)
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// The path as a `String`, terminator dropped.
    pub fn into_string(mut self) -> String {
        self.inner.pop();
        self.inner
    }

    /// Appends the part of `s` a C caller could see, leaving the terminator to
    /// the caller. Expects the existing one to have been popped or truncated
    /// away.
    fn push_raw(&mut self, s: &str) {
        let s = match s.find('\0') {
            Some(i) => &s[..i],
            None => s,
        };

        self.inner.push_str(s);
    }
}

/// Iterator over the components of a [`Path`].
#[derive(Clone)]
pub struct Components<'a> {
    rest: &'a str,
}

impl<'a> Components<'a> {
    /// The part of the path not yet yielded, with no leading separator.
    #[inline]
    pub fn as_path(&self) -> &'a Path {
        Path::new(self.rest)
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        loop {
            let b = self.rest.as_bytes();

            let mut end = 0;
            while end < b.len() && !is_sep(b[end]) {
                end += 1;
            }

            // Eat the separators after the component too, so `rest` is always
            // a path in its own right.
            let mut next = end;
            while next < b.len() && is_sep(b[next]) {
                next += 1;
            }

            let comp = &self.rest[..end];
            self.rest = &self.rest[next..];

            if comp.is_empty() {
                if next == end {
                    return None;
                }

                continue;
            }

            if comp != "." {
                return Some(comp);
            }
        }
    }
}

impl Default for PathBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for PathBuf {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Path {
        self.as_path()
    }
}

impl Borrow<Path> for PathBuf {
    #[inline]
    fn borrow(&self) -> &Path {
        self.as_path()
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;

    fn to_owned(&self) -> PathBuf {
        PathBuf::from(self)
    }
}

impl AsRef<Path> for Path {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<Path> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<Path> for str {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl AsRef<str> for Path {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&Path> for PathBuf {
    fn from(path: &Path) -> Self {
        let s = path.as_str();
        let s = match s.find('\0') {
            Some(i) => &s[..i],
            None => s,
        };

        let mut inner = String::with_capacity(s.len() + 1);
        inner.push_str(s);
        inner.push('\0');

        Self { inner }
    }
}

impl From<&str> for PathBuf {
    fn from(s: &str) -> Self {
        PathBuf::from(Path::new(s))
    }
}

impl From<String> for PathBuf {
    fn from(mut s: String) -> Self {
        if let Some(i) = s.find('\0') {
            s.truncate(i);
        }

        s.push('\0');

        Self { inner: s }
    }
}

impl From<PathBuf> for String {
    fn from(path: PathBuf) -> String {
        path.into_string()
    }
}

impl<P: AsRef<Path>> FromIterator<P> for PathBuf {
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut buf = PathBuf::new();

        for part in iter {
            buf.push(part);
        }

        buf
    }
}

impl<P: AsRef<Path>> Extend<P> for PathBuf {
    fn extend<I: IntoIterator<Item = P>>(&mut self, iter: I) {
        for part in iter {
            self.push(part);
        }
    }
}

// Comparison is per component, so `a/b`, `a//b` and `./a/b` are one path.
// Rootedness is compared separately because `Components` drops it.

impl PartialEq for Path {
    fn eq(&self, other: &Path) -> bool {
        self.is_absolute() == other.is_absolute() && self.components().eq(other.components())
    }
}

impl Eq for Path {}

impl Hash for Path {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_absolute().hash(state);

        for comp in self.components() {
            comp.hash(state);
        }
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Path) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Path) -> Ordering {
        self.is_absolute()
            .cmp(&other.is_absolute())
            .then_with(|| self.components().cmp(other.components()))
    }
}

impl PartialEq for PathBuf {
    fn eq(&self, other: &PathBuf) -> bool {
        **self == **other
    }
}

impl Eq for PathBuf {}

impl Hash for PathBuf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl PartialOrd for PathBuf {
    fn partial_cmp(&self, other: &PathBuf) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathBuf {
    fn cmp(&self, other: &PathBuf) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl PartialEq<Path> for PathBuf {
    fn eq(&self, other: &Path) -> bool {
        **self == *other
    }
}

impl PartialEq<PathBuf> for Path {
    fn eq(&self, other: &PathBuf) -> bool {
        *self == **other
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;

    fn comps(p: &str) -> Vec<&str> {
        Path::new(p).components().collect()
    }

    #[test]
    fn components() {
        assert_eq!(comps(""), [] as [&str; 0]);
        assert_eq!(comps("/"), [] as [&str; 0]);
        assert_eq!(comps("a"), ["a"]);
        assert_eq!(comps("/a/b"), ["a", "b"]);
        assert_eq!(comps("a//b/"), ["a", "b"]);
        assert_eq!(comps("a/./b"), ["a", "b"]);
        assert_eq!(comps("a\\b"), ["a", "b"]);
        assert_eq!(comps("../a"), ["..", "a"]);
    }

    #[test]
    fn file_name() {
        assert_eq!(Path::new("a/b.png").file_name(), Some("b.png"));
        assert_eq!(Path::new("a/b/").file_name(), Some("b"));
        assert_eq!(Path::new("b").file_name(), Some("b"));
        assert_eq!(Path::new("/").file_name(), None);
        assert_eq!(Path::new("").file_name(), None);
        assert_eq!(Path::new("a/..").file_name(), None);
    }

    #[test]
    fn extensions() {
        assert_eq!(Path::new("a/b.png").extension(), Some("png"));
        assert_eq!(Path::new("a/b.tar.gz").extension(), Some("gz"));
        assert_eq!(Path::new("a/b").extension(), None);
        assert_eq!(Path::new("a/.gitignore").extension(), None);
        assert_eq!(Path::new("a/.gitignore").file_stem(), Some(".gitignore"));
        assert_eq!(Path::new("a/b.tar.gz").file_stem(), Some("b.tar"));
    }

    #[test]
    fn parents() {
        assert_eq!(Path::new("a/b/c").parent().unwrap().as_str(), "a/b");
        assert_eq!(Path::new("a//b").parent().unwrap().as_str(), "a");
        assert_eq!(Path::new("a/b/").parent().unwrap().as_str(), "a");
        assert_eq!(Path::new("/a").parent().unwrap().as_str(), "/");
        assert_eq!(Path::new("a").parent().unwrap().as_str(), "");
        assert!(Path::new("/").parent().is_none());
        assert!(Path::new("///").parent().is_none());
        assert!(Path::new("").parent().is_none());
    }

    #[test]
    fn joining() {
        assert_eq!(Path::new("assets").join("a.png").as_str(), "assets/a.png");
        assert_eq!(Path::new("assets/").join("a.png").as_str(), "assets/a.png");
        assert_eq!(Path::new("").join("a.png").as_str(), "a.png");
        assert_eq!(Path::new("assets").join("").as_str(), "assets");
        assert_eq!(Path::new("assets").join("/etc").as_str(), "/etc");
        assert_eq!(Path::new("a").join("b").join("c").as_str(), "a/b/c");
    }

    #[test]
    fn pushing_and_popping() {
        let mut buf = PathBuf::from("assets");

        buf.push("img");
        buf.push("a.png");
        assert_eq!(buf.as_str(), "assets/img/a.png");

        assert!(buf.pop());
        assert_eq!(buf.as_str(), "assets/img");

        assert!(buf.pop());
        assert!(buf.pop());
        assert_eq!(buf.as_str(), "");
        assert!(!buf.pop());
    }

    #[test]
    fn extension_edits() {
        assert_eq!(
            Path::new("a/b.png").with_extension("qoi").as_str(),
            "a/b.qoi"
        );
        assert_eq!(Path::new("a/b").with_extension("png").as_str(), "a/b.png");
        assert_eq!(Path::new("a/b.png").with_extension("").as_str(), "a/b");
        assert_eq!(Path::new("a/b/").with_extension("png").as_str(), "a/b.png");
        assert!(!PathBuf::from("/").set_extension("png"));

        assert_eq!(
            Path::new("a/b.png").with_file_name("c.qoi").as_str(),
            "a/c.qoi"
        );
        assert_eq!(Path::new("").with_file_name("c").as_str(), "c");
    }

    #[test]
    fn prefixes() {
        let p = Path::new("assets/img/a.png");

        assert_eq!(p.strip_prefix("assets").unwrap().as_str(), "img/a.png");
        assert_eq!(p.strip_prefix("assets//img/").unwrap().as_str(), "a.png");
        assert_eq!(p.strip_prefix("").unwrap().as_str(), "assets/img/a.png");
        assert!(p.strip_prefix("img").is_none());
        assert!(p.strip_prefix("/assets").is_none());
        assert!(p.starts_with("assets/img"));
    }

    #[test]
    fn equality_ignores_separator_noise() {
        assert_eq!(Path::new("a/b"), Path::new("a//b"));
        assert_eq!(Path::new("a/b"), Path::new("./a/b"));
        assert_ne!(Path::new("a/b"), Path::new("/a/b"));
        assert_ne!(Path::new("ab"), Path::new("a/b"));
        assert_eq!(PathBuf::from("a/b/"), PathBuf::from("a/b"));
    }

    #[test]
    fn terminator_is_hidden_but_present() {
        let buf = PathBuf::from("a/b.png");

        assert_eq!(buf.as_str(), "a/b.png");
        assert_eq!(buf.inner.as_bytes(), b"a/b.png\0");
        assert_eq!(&*buf.into_string(), "a/b.png");

        // An interior NUL is dropped rather than left to truncate the path
        // behind a C caller's back.
        assert_eq!(PathBuf::from("a\0b").as_str(), "a");
        assert_eq!(PathBuf::from(String::from("a\0b")).as_str(), "a");

        let mut buf = PathBuf::from("a");
        buf.push("b\0c");
        assert_eq!(buf.as_str(), "a/b");
    }

    #[test]
    fn c_str_roundtrip() {
        fn as_bytes(p: &Path) -> Vec<u8> {
            p.with_c_str(|ptr| {
                let mut out = Vec::new();
                let mut i = 0;

                loop {
                    let b = unsafe { *ptr.add(i) } as u8;
                    if b == 0 {
                        break out;
                    }
                    out.push(b);
                    i += 1;
                }
            })
        }

        assert_eq!(as_bytes(Path::new("a/b.png")), b"a/b.png");
        assert_eq!(as_bytes(Path::new("")), b"");

        let long: String = core::iter::repeat('x').take(INLINE_C_STR + 10).collect();
        assert_eq!(as_bytes(Path::new(&long)).len(), INLINE_C_STR + 10);
    }
}

extern crate alloc;

use alloc::string::String;

pub const COMPILED_VERSION: (u32, u32, u32) = (
    parse_env(env!("KARNA_SDL_IMAGE_MAJOR")),
    parse_env(env!("KARNA_SDL_IMAGE_MINOR")),
    parse_env(env!("KARNA_SDL_IMAGE_MICRO")),
);

pub fn compiled_version_string() -> String {
    let (major, minor, micro) = COMPILED_VERSION;
    alloc::format!("{major}.{minor}.{micro}")
}

const fn parse_env(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut acc = 0u32;
    let mut i = 0;
    while i < bytes.len() {
        acc = acc * 10 + (bytes[i] - b'0') as u32;
        i += 1;
    }
    acc
}

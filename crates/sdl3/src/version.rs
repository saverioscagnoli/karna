extern crate alloc;

use alloc::string::String;

use crate::SDL_GetVersion;

pub const COMPILED_VERSION: (u32, u32, u32) = (
    parse_env(env!("KARNA_SDL_MAJOR")),
    parse_env(env!("KARNA_SDL_MINOR")),
    parse_env(env!("KARNA_SDL_MICRO")),
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

pub const fn version_num(major: u32, minor: u32, micro: u32) -> i32 {
    (major * 1_000_000 + minor * 1_000 + micro) as i32
}

pub unsafe fn check_linked_version() -> Result<i32, i32> {
    let runtime = unsafe { SDL_GetVersion() };
    let (maj, min, mic) = COMPILED_VERSION;
    let compiled = version_num(maj, min, mic);

    if runtime < compiled {
        Err(runtime)
    } else {
        Ok(runtime)
    }
}

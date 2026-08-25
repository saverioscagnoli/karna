//! Raw FFI bindings to SDL3_image.

#![no_std]
#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    improper_ctypes,
    clippy::all
)]

// The generated bindings pull core SDL3 types (SDL_Surface, SDL_Renderer,
// ...) in via `use sdl3::*;` at their top, rather than redefining them.
include!(concat!(env!("OUT_DIR"), "/sdl3_image.rs"));

mod version;
pub use version::COMPILED_VERSION;
pub use version::compiled_version_string;

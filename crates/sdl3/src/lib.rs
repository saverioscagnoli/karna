//! Raw FFI bindings to SDL3.
//!
//! Nothing here is safe. This crate exists only to expose the C API; all
//! invariants (main-thread affinity, init ordering, pointer lifetimes) are
//! the caller's problem. Build the safe layer in `engine`.

#![no_std]
#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    improper_ctypes,
    // bindgen derives PartialEq/Eq/Hash on SDL's function-pointer interface
    // structs (e.g. SDL_StorageInterface); we never rely on that comparison
    // being meaningful, so the lint has nothing useful to say here.
    unpredictable_function_pointer_comparisons,
    clippy::all
)]

include!(concat!(env!("OUT_DIR"), "/sdl3.rs"));

mod version;
pub use version::{check_linked_version, compiled_version_string, version_num, COMPILED_VERSION};

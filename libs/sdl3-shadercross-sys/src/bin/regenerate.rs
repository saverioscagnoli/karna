//! Regenerates `src/bindings.rs` from the vendored SDL_shadercross headers.
//!
//! This is a developer tool, not part of a normal build: the generated file is
//! committed so that users never need libclang installed.
//!
//!     cargo run -p karna-sdl3-shadercross-sys --features regenerate,build-from-source --bin regenerate-bindings

use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdl_include = manifest.join("../../vendor/SDL/include");
    let shadercross_include = manifest.join("../../vendor/SDL_shadercross/include");

    assert!(
        sdl_include.join("SDL3/SDL.h").exists(),
        "vendored SDL headers not found at {}\n\
         Run: git submodule update --init --depth 1 vendor/SDL",
        sdl_include.display()
    );
    assert!(
        shadercross_include.join("SDL3_shadercross/SDL_shadercross.h").exists(),
        "vendored SDL_shadercross headers not found at {}\n\
         Run: git submodule update --init --depth 1 vendor/SDL_shadercross",
        shadercross_include.display()
    );

    let out = manifest.join("src/bindings.rs");

    let bindings = bindgen::Builder::default()
        .header(manifest.join("wrapper.h").to_str().unwrap())
        .clang_arg(format!("-I{}", sdl_include.display()))
        .clang_arg(format!("-I{}", shadercross_include.display()))
        // Map C types to core::ffi so the committed file stays correct on
        // targets where c_char signedness or c_long width differ.
        .use_core()
        .ctypes_prefix("core::ffi")
        // Layout tests bake in the host's struct sizes, which would break the
        // committed bindings on other targets.
        .layout_tests(false)
        .allowlist_function("SDL_ShaderCross_.*")
        .allowlist_type("SDL_ShaderCross_.*")
        .allowlist_var("SDL_SHADERCROSS_.*")
        // Don't pull in the SDL types that SDL_ShaderCross_* items refer to; lib.rs
        // imports them from sdl3_sys instead, so they are the same types.
        .allowlist_recursively(false)
        .derive_debug(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .generate()
        .expect("bindgen failed");

    bindings.write_to_file(&out).expect("failed to write bindings");
    println!("wrote {}", out.display());
}

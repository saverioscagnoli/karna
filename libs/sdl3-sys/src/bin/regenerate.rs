//! Regenerates `src/bindings.rs` from the vendored SDL headers.
//!
//! This is a developer tool, not part of a normal build: the generated file is
//! committed so that users never need libclang installed.
//!
//!     cargo run -p karna-sdl3-sys --features regenerate --bin regenerate-bindings

use std::path::Path;

use bindgen::callbacks::ItemInfo;
use bindgen::callbacks::ParseCallbacks;

/// Drops the `karna_` prefix that `wrapper.h` puts on the window flag
/// constants, so the generated names match SDL's own.
#[derive(Debug)]
struct StripKarnaPrefix;

impl ParseCallbacks for StripKarnaPrefix {
    fn item_name(&self, item: ItemInfo) -> Option<String> {
        item.name.strip_prefix("karna_").map(String::from)
    }
}

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let include = manifest.join("../../vendor/SDL/include");

    assert!(
        include.join("SDL3/SDL.h").exists(),
        "vendored SDL headers not found at {}\n\
         Run: git submodule update --init --depth 1 vendor/SDL",
        include.display()
    );

    let out = manifest.join("src/bindings.rs");

    let bindings = bindgen::Builder::default()
        .header(manifest.join("wrapper.h").to_str().unwrap())
        .clang_arg(format!("-I{}", include.display()))
        // Map C types to core::ffi so the committed file stays correct on
        // targets where c_char signedness or c_long width differ.
        .use_core()
        .ctypes_prefix("core::ffi")
        // Layout tests bake in the host's struct sizes, which would break the
        // committed bindings on other targets.
        .layout_tests(false)
        .allowlist_function("SDL_.*")
        .allowlist_type("SDL_.*")
        .allowlist_var("SDL_.*")
        .allowlist_var("SDLK_.*")
        // The window flag shims in wrapper.h; StripKarnaPrefix renames them
        // back to SDL_WINDOW_*.
        .allowlist_var("karna_SDL_.*")
        .parse_callbacks(Box::new(StripKarnaPrefix))
        .derive_debug(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .generate()
        .expect("bindgen failed");

    bindings.write_to_file(&out).expect("failed to write bindings");
    println!("wrote {}", out.display());
}

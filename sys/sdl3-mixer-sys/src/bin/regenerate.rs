use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdl_include = manifest.join("../../vendor/SDL/include");
    let mixer_include = manifest.join("../../vendor/SDL_mixer/include");

    assert!(
        sdl_include.join("SDL3/SDL.h").exists(),
        "vendored SDL headers not found at {}\n\
         Run: git submodule update --init --depth 1 vendor/SDL",
        sdl_include.display()
    );
    assert!(
        mixer_include.join("SDL3_mixer/SDL_mixer.h").exists(),
        "vendored SDL_mixer headers not found at {}\n\
         Run: git submodule update --init --depth 1 vendor/SDL_mixer",
        mixer_include.display()
    );

    let out = manifest.join("src/bindings.rs");

    let bindings = bindgen::Builder::default()
        .header(manifest.join("wrapper.h").to_str().unwrap())
        .clang_arg(format!("-I{}", sdl_include.display()))
        .clang_arg(format!("-I{}", mixer_include.display()))
        .use_core()
        .ctypes_prefix("core::ffi")
        .layout_tests(false)
        .allowlist_function("MIX_.*")
        .allowlist_type("MIX_.*")
        .allowlist_var("SDL_MIXER_.*")
        .allowlist_var("MIX_.*")
        .allowlist_recursively(false)
        .derive_debug(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .generate()
        .expect("bindgen failed");

    bindings.write_to_file(&out).expect("failed to write bindings");
    println!("wrote {}", out.display());
}

#[cfg(feature = "build-from-source")]
use std::path::Path;
#[cfg(feature = "build-from-source")]
use std::path::PathBuf;

#[cfg(feature = "build-from-source")]
fn vendored_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/SDL_image")
}

/// Link against an SDL3_image already installed on the system.
fn link_system() {
    match pkg_config::Config::new()
        .atleast_version("3.2")
        .probe("sdl3-image")
    {
        Ok(_) => {}
        Err(e) => panic!(
            "could not find a system SDL3_image via pkg-config: {e}\n\
             Either install SDL3_image development files, or enable the \
             `build-from-source` feature to build the vendored copy."
        ),
    }
}

/// Build the vendored SDL_image source tree with CMake and link the result.
#[cfg(feature = "build-from-source")]
fn build_vendored() {
    let src = vendored_dir();
    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "vendored SDL_image not found at {}\n\
             The submodule is not checked out. Run:\n    \
             git submodule update --init --depth 1 vendor/SDL_image",
            src.display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        src.join("CMakeLists.txt").display()
    );

    let mut config = cmake::Config::new(&src);

    // Point CMake's find_package(SDL3) at the SDL that karna-sdl3-sys built,
    // so we don't pick up a different system copy.
    if let Ok(sdl_root) = std::env::var("DEP_SDL3_ROOT") {
        config.define("CMAKE_PREFIX_PATH", sdl_root);
    }

    let dst = config
        .define("BUILD_SHARED_LIBS", "ON")
        .define("SDLIMAGE_SAMPLES", "OFF")
        .define("SDLIMAGE_TESTS", "OFF")
        // The third-party codec submodules under external/ are not checked
        // out. PNG and JPG go through the bundled stb_image instead, and the
        // formats that have no built-in decoder are disabled.
        .define("SDLIMAGE_VENDORED", "OFF")
        .define("SDLIMAGE_BACKEND_STB", "ON")
        .define("SDLIMAGE_PNG_LIBPNG", "OFF")
        .define("SDLIMAGE_AVIF", "OFF")
        .define("SDLIMAGE_JXL", "OFF")
        .define("SDLIMAGE_TIF", "OFF")
        .define("SDLIMAGE_WEBP", "OFF")
        .build();

    // Installs to lib/ or lib64/ depending on the platform.
    for dir in ["lib", "lib64"] {
        let path = dst.join(dir);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
            // So test binaries and examples can find libSDL3_image.so without
            // the caller setting LD_LIBRARY_PATH.
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        }
    }
    println!("cargo:rustc-link-lib=dylib=SDL3_image");
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}", dst.join("include").display());
}

#[cfg(not(feature = "build-from-source"))]
fn build_vendored() {
    unreachable!()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    if cfg!(feature = "build-from-source") {
        build_vendored();
    } else {
        link_system();
    }
}

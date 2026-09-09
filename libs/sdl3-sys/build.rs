#[cfg(feature = "build-from-source")]
use std::path::Path;
#[cfg(feature = "build-from-source")]
use std::path::PathBuf;

#[cfg(feature = "build-from-source")]
fn vendored_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/SDL")
}

/// Link against an SDL3 already installed on the system.
fn link_system() {
    match pkg_config::Config::new()
        .atleast_version("3.2")
        .probe("sdl3")
    {
        Ok(_) => {}
        Err(e) => panic!(
            "could not find a system SDL3 via pkg-config: {e}\n\
             Either install SDL3 development files, or enable the \
             `build-from-source` feature to build the vendored copy."
        ),
    }
}

/// Build the vendored SDL source tree with CMake and link the result.
#[cfg(feature = "build-from-source")]
fn build_vendored() {
    let src = vendored_dir();
    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "vendored SDL not found at {}\n\
             The submodule is not checked out. Run:\n    \
             git submodule update --init --depth 1 vendor/SDL",
            src.display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        src.join("CMakeLists.txt").display()
    );

    let dst = cmake::Config::new(&src)
        .define("SDL_SHARED", "ON")
        .define("SDL_STATIC", "OFF")
        .define("SDL_TESTS", "OFF")
        .define("SDL_EXAMPLES", "OFF")
        .define("SDL_INSTALL_TESTS", "OFF")
        .build();

    // SDL installs to lib/ or lib64/ depending on the platform.
    for dir in ["lib", "lib64"] {
        let path = dst.join(dir);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
            // So test binaries and examples can find libSDL3.so without
            // the caller setting LD_LIBRARY_PATH.
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        }
    }
    println!("cargo:rustc-link-lib=dylib=SDL3");
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

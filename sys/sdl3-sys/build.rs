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

/// Copy the DLLs CMake installed into `<dst>/bin` next to the executables
/// cargo is about to produce.
///
/// Windows resolves DLLs from the directory of the running binary, so without
/// this every `cargo run` / `cargo test` would fail to start. OUT_DIR is
/// `<target>/<profile>/build/<pkg>-<hash>/out`, so the profile directory --
/// where cargo puts the final binaries -- is four levels up.
#[cfg(feature = "build-from-source")]
fn copy_runtime_dlls(dst: &Path) {
    let bin = dst.join("bin");
    let Ok(entries) = std::fs::read_dir(&bin) else {
        return;
    };

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let Some(profile_dir) = out_dir.ancestors().nth(3) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("dll")) {
            let name = entry.file_name();
            // Also into deps/, which is where test and example binaries run from.
            for dir in [profile_dir.to_path_buf(), profile_dir.join("deps")] {
                if dir.is_dir() {
                    let _ = std::fs::copy(&path, dir.join(&name));
                }
            }
        }
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

    let windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");

    // SDL installs to lib/ or lib64/ depending on the platform.
    for dir in ["lib", "lib64"] {
        let path = dst.join(dir);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
            // So test binaries and examples can find libSDL3.so without
            // the caller setting LD_LIBRARY_PATH. Windows has no rpath (and
            // the MSVC linker rejects the flag), so the DLL is copied next to
            // the executable instead.
            if !windows {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
            }
        }
    }
    if windows {
        copy_runtime_dlls(&dst);
    }
    println!("cargo:rustc-link-lib=dylib=SDL3");
    // Exposed to dependents as DEP_SDL3_ROOT; karna-sdl3-image-sys hands it
    // to CMake so SDL_image builds against this copy of SDL.
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

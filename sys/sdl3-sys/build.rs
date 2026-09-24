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

#[cfg(feature = "build-from-source")]
fn link_private_libs(pc: &Path) {
    let Ok(contents) = std::fs::read_to_string(pc) else {
        return;
    };

    let mut tokens = contents
        .lines()
        .filter_map(|line| line.strip_prefix("Libs:").or_else(|| line.strip_prefix("Libs.private:")))
        .flat_map(str::split_whitespace);

    while let Some(token) = tokens.next() {
        if token == "-framework" {
            if let Some(name) = tokens.next() {
                println!("cargo:rustc-link-lib=framework={name}");
            }
        } else if let Some(name) = token.strip_prefix("-Wl,-framework,") {
            println!("cargo:rustc-link-lib=framework={name}");
        } else if token == "-pthread" {
            println!("cargo:rustc-link-lib=pthread");
        } else if let Some(name) = token.strip_prefix("-l") {
            if name != "SDL3" {
                println!("cargo:rustc-link-lib={name}");
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
        .define("SDL_SHARED", "OFF")
        .define("SDL_STATIC", "ON")
        .define("SDL_TESTS", "OFF")
        .define("SDL_EXAMPLES", "OFF")
        .define("SDL_INSTALL_TESTS", "OFF")
        .build();

    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");

    // SDL installs to lib/ or lib64/ depending on the platform.
    for dir in ["lib", "lib64"] {
        let path = dst.join(dir);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
    }
    println!(
        "cargo:rustc-link-lib=static={}",
        if msvc { "SDL3-static" } else { "SDL3" }
    );
    for dir in ["lib", "lib64"] {
        link_private_libs(&dst.join(dir).join("pkgconfig/sdl3.pc"));
    }
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

use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "build-from-source")]
fn vendored_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/quickjs")
}

/// Link against a quickjs-ng already installed on the system, returning the
/// include directories for its headers.
fn link_system() -> Vec<PathBuf> {
    match pkg_config::Config::new()
        .atleast_version("0.17")
        .probe("quickjs-ng")
    {
        Ok(lib) => lib.include_paths,
        Err(e) => panic!(
            "could not find a system quickjs-ng via pkg-config: {e}\n\
             Either install quickjs-ng development files, or enable the \
             `build-from-source` feature to build the vendored copy."
        ),
    }
}

/// Build the vendored QuickJS source tree with CMake as a static library and
/// link the result, returning the include directory for its headers.
#[cfg(feature = "build-from-source")]
fn build_vendored() -> Vec<PathBuf> {
    let src = vendored_dir();
    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "vendored QuickJS not found at {}\n\
             The submodule is not checked out. Run:\n    \
             git submodule update --init --depth 1 vendor/quickjs",
            src.display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        src.join("CMakeLists.txt").display()
    );

    // Only the library target is built: installing would also build the qjs
    // and qjsc executables, which nothing here needs.
    let dst = cmake::Config::new(&src)
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define("QJS_BUILD_EXAMPLES", "OFF")
        .define("QJS_BUILD_LIBC", "OFF")
        .build_target("qjs")
        .build();

    // Single-config generators put the archive in build/, multi-config ones
    // (Visual Studio, Xcode) in build/<config>/.
    let build = dst.join("build");
    for dir in ["", "Debug", "Release", "RelWithDebInfo", "MinSizeRel"] {
        let path = build.join(dir);
        if path.is_dir() {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
    }
    println!("cargo:rustc-link-lib=static=qjs");

    // The static archive doesn't carry its own dependencies; these are the
    // ones CMakeLists.txt links qjs against.
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => {}
        Ok("linux" | "android") => {
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=pthread");
        }
        _ => println!("cargo:rustc-link-lib=m"),
    }

    println!("cargo:root={}", dst.display());
    println!("cargo:include={}", src.display());
    vec![src]
}

#[cfg(not(feature = "build-from-source"))]
fn build_vendored() -> Vec<PathBuf> {
    unreachable!()
}

/// Compile the bindgen-generated shim that gives quickjs.h's static inline
/// functions real symbols to link against.
fn build_shim(includes: &[PathBuf]) {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shim = manifest.join("src/extern.c");
    println!("cargo:rerun-if-changed={}", shim.display());

    let mut build = cc::Build::new();
    build.file(&shim).include(manifest).includes(includes);
    // quickjs.h relies on C11 (stdbool, anonymous unions, compound literals).
    if build.get_compiler().is_like_msvc() {
        build.flag("/std:c11");
    } else {
        build.flag("-std=c11");
    }
    build.compile("quickjs_sys_extern");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    let includes = if cfg!(feature = "build-from-source") {
        build_vendored()
    } else {
        link_system()
    };
    build_shim(&includes);
}

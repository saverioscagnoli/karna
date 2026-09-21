#[cfg(feature = "build-from-source")]
use std::path::Path;
#[cfg(feature = "build-from-source")]
use std::path::PathBuf;

#[cfg(feature = "build-from-source")]
fn vendored_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/SDL_shadercross")
}

/// Link against an SDL3_shadercross already installed on the system.
fn link_system() {
    match pkg_config::Config::new()
        .atleast_version("3.0")
        .probe("sdl3-shadercross")
    {
        Ok(_) => {}
        Err(e) => panic!(
            "could not find a system SDL3_shadercross via pkg-config: {e}\n\
             Either install SDL3_shadercross development files, or enable the \
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

/// Build the SPIRV-Cross submodule nested in SDL_shadercross as static,
/// position-independent libraries, so they can be folded into the shared
/// SDL3_shadercross.
///
/// SDL_shadercross's own SDLSHADERCROSS_VENDORED mode is not used: it insists
/// on SPIRV-Tools, SPIRV-Headers and DirectXShaderCompiler being checked out
/// too, and DXC alone is a multi-gigabyte LLVM fork. Without DXC, SPIR-V still
/// transpiles to MSL and HLSL, and on Windows HLSL compiles to DXBC through
/// the system d3dcompiler_47.dll, which D3D12 accepts.
#[cfg(feature = "build-from-source")]
fn build_spirv_cross(src: &Path, out: &Path) -> PathBuf {
    let spirv = src.join("external/SPIRV-Cross");
    if !spirv.join("CMakeLists.txt").exists() {
        panic!(
            "SPIRV-Cross not found at {}\n\
             The nested submodule is not checked out. Run:\n    \
             git -C vendor/SDL_shadercross submodule update --init --depth 1 external/SPIRV-Cross",
            spirv.display()
        );
    }

    cmake::Config::new(&spirv)
        .out_dir(out.join("spirv-cross"))
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define("SPIRV_CROSS_STATIC", "ON")
        .define("SPIRV_CROSS_SHARED", "OFF")
        .define("SPIRV_CROSS_CLI", "OFF")
        .define("SPIRV_CROSS_ENABLE_TESTS", "OFF")
        .define("SPIRV_CROSS_ENABLE_GLSL", "ON")
        .define("SPIRV_CROSS_ENABLE_HLSL", "ON")
        .define("SPIRV_CROSS_ENABLE_MSL", "ON")
        .define("SPIRV_CROSS_ENABLE_CPP", "ON")
        .define("SPIRV_CROSS_ENABLE_REFLECT", "ON")
        .define("SPIRV_CROSS_ENABLE_C_API", "ON")
        .define("SPIRV_CROSS_ENABLE_UTIL", "ON")
        .build()
}

/// Build the vendored SDL_shadercross source tree with CMake and link the
/// result.
#[cfg(feature = "build-from-source")]
fn build_vendored() {
    let src = vendored_dir();
    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "vendored SDL_shadercross not found at {}\n\
             The submodule is not checked out. Run:\n    \
             git submodule update --init --depth 1 vendor/SDL_shadercross",
            src.display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        src.join("CMakeLists.txt").display()
    );

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let spirv_root = build_spirv_cross(&src, &out);

    // Point CMake's find_package(SDL3) at the SDL that karna-sdl3-sys built,
    // and find_package(spirv_cross_*) at the copy built above.
    // CMake lists are ';'-separated on every platform.
    let mut prefix = spirv_root.display().to_string();
    if let Ok(sdl_root) = std::env::var("DEP_SDL3_ROOT") {
        prefix.push(';');
        prefix.push_str(&sdl_root);
    }

    let dst = cmake::Config::new(&src)
        .out_dir(out.join("shadercross"))
        .define("CMAKE_PREFIX_PATH", prefix)
        .define("SDLSHADERCROSS_SHARED", "ON")
        .define("SDLSHADERCROSS_STATIC", "OFF")
        .define("SDLSHADERCROSS_VENDORED", "OFF")
        .define("SDLSHADERCROSS_SPIRVCROSS_SHARED", "OFF")
        .define("SDLSHADERCROSS_DXC", "OFF")
        .define("SDLSHADERCROSS_CLI", "OFF")
        .define("SDLSHADERCROSS_TESTS", "OFF")
        .define("SDLSHADERCROSS_INSTALL", "ON")
        .build();

    let windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");

    // Installs to lib/ or lib64/ depending on the platform.
    for dir in ["lib", "lib64"] {
        let path = dst.join(dir);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
            // So test binaries and examples can find libSDL3_shadercross.so
            // without the caller setting LD_LIBRARY_PATH. Windows has no rpath
            // (and the MSVC linker rejects the flag), so the DLL is copied next
            // to the executable instead.
            if !windows {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
            }
        }
    }
    if windows {
        copy_runtime_dlls(&dst);
    }
    println!("cargo:rustc-link-lib=dylib=SDL3_shadercross");
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

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
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dll"))
        {
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

#[cfg(feature = "build-from-source")]
fn require_submodule(src: &Path, path: &str) {
    if !src.join(path).join("CMakeLists.txt").exists() {
        panic!(
            "{} not found at {}\n\
             The vendored submodules are not checked out. Run:\n    \
             git submodule update --init --recursive vendor/SDL_shadercross",
            path,
            src.join(path).display()
        );
    }
}

#[cfg(feature = "dxc")]
fn collect_dxc(build: &Path, dst: &Path, windows: bool) {
    let target = dst.join(if windows { "bin" } else { "lib" });
    let _ = std::fs::create_dir_all(&target);

    let wanted = |name: &str| {
        if windows {
            name.eq_ignore_ascii_case("dxcompiler.dll") || name.eq_ignore_ascii_case("dxil.dll")
        } else {
            ["libdxcompiler.", "libdxil."]
                .iter()
                .any(|p| name.starts_with(p))
                && (name.contains(".so") || name.ends_with(".dylib"))
        }
    };

    let mut stack = vec![build.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();

            if path.is_dir() {
                stack.push(path);
            } else if wanted(&name) {
                let _ = std::fs::copy(&path, target.join(&name));
            }
        }
    }
}

#[cfg(feature = "dxc")]
fn build_shadercross(src: &Path, shared: &Path, origin: &str, windows: bool) -> PathBuf {
    for path in [
        "external/SPIRV-Cross",
        "external/SPIRV-Headers",
        "external/SPIRV-Tools",
        "external/DirectXShaderCompiler",
    ] {
        require_submodule(src, path);
    }

    let mut config = cmake::Config::new(src);

    config
        .out_dir(shared.join("shadercross"))
        .profile("Release")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define("CMAKE_INSTALL_RPATH", origin)
        .define("SDLSHADERCROSS_SHARED", "ON")
        .define("SDLSHADERCROSS_STATIC", "OFF")
        .define("SDLSHADERCROSS_VENDORED", "ON")
        .define("SDLSHADERCROSS_SPIRVCROSS_SHARED", "OFF")
        .define("SDLSHADERCROSS_DXC", "ON")
        .define("SDLSHADERCROSS_CLI", "OFF")
        .define("SDLSHADERCROSS_TESTS", "OFF")
        .define("SDLSHADERCROSS_INSTALL", "ON")
        .define("LLVM_ENABLE_ZLIB", "OFF");

    if let Ok(sdl_root) = std::env::var("DEP_SDL3_ROOT") {
        config.define("CMAKE_PREFIX_PATH", sdl_root);
    }

    let dst = config.build();

    collect_dxc(&dst.join("build"), &dst, windows);

    dst
}

#[cfg(all(feature = "build-from-source", not(feature = "dxc")))]
fn build_shadercross(src: &Path, shared: &Path, origin: &str, _windows: bool) -> PathBuf {
    require_submodule(src, "external/SPIRV-Cross");

    let spirv_root = cmake::Config::new(src.join("external/SPIRV-Cross"))
        .out_dir(shared.join("spirv-cross"))
        .profile("Release")
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
        .build();

    let mut prefix = spirv_root.display().to_string();
    if let Ok(sdl_root) = std::env::var("DEP_SDL3_ROOT") {
        prefix.push(';');
        prefix.push_str(&sdl_root);
    }

    cmake::Config::new(src)
        .out_dir(shared.join("shadercross-nodxc"))
        .profile("Release")
        .define("CMAKE_PREFIX_PATH", prefix)
        .define("CMAKE_INSTALL_RPATH", origin)
        .define("SDLSHADERCROSS_SHARED", "ON")
        .define("SDLSHADERCROSS_STATIC", "OFF")
        .define("SDLSHADERCROSS_VENDORED", "OFF")
        .define("SDLSHADERCROSS_SPIRVCROSS_SHARED", "OFF")
        .define("SDLSHADERCROSS_DXC", "OFF")
        .define("SDLSHADERCROSS_CLI", "OFF")
        .define("SDLSHADERCROSS_TESTS", "OFF")
        .define("SDLSHADERCROSS_INSTALL", "ON")
        .build()
}

#[cfg(feature = "build-from-source")]
fn build_vendored() {
    let src = vendored_dir();
    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "vendored SDL_shadercross not found at {}\n\
             The submodule is not checked out. Run:\n    \
             git submodule update --init --recursive vendor/SDL_shadercross",
            src.display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        src.join("CMakeLists.txt").display()
    );

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let shared = out
        .ancestors()
        .find(|dir| dir.file_name().is_some_and(|name| name == "build"))
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map_or_else(|| out.clone(), |target| target.join("karna-vendor"));
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let windows = target_os == "windows";
    let origin = if target_os == "macos" || target_os == "ios" {
        "@loader_path"
    } else {
        "$ORIGIN"
    };

    let dst = build_shadercross(&src, &shared, origin, windows);

    for dir in ["lib", "lib64"] {
        let path = dst.join(dir);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
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

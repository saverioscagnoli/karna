#[cfg(feature = "build-from-source")]
use std::path::Path;
#[cfg(feature = "build-from-source")]
use std::path::PathBuf;

#[cfg(feature = "build-from-source")]
fn vendored_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/SDL_mixer")
}

fn link_system() {
    match pkg_config::Config::new()
        .atleast_version("3.2")
        .probe("sdl3-mixer")
    {
        Ok(_) => {}
        Err(e) => panic!(
            "could not find a system SDL3_mixer via pkg-config: {e}\n\
             Either install SDL3_mixer development files, or enable the \
             `build-from-source` feature to build the vendored copy."
        ),
    }
}

#[cfg(feature = "build-from-source")]
fn build_vendored() {
    let src = vendored_dir();
    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "vendored SDL_mixer not found at {}\n\
             The submodule is not checked out. Run:\n    \
             git submodule update --init --depth 1 vendor/SDL_mixer",
            src.display()
        );
    }

    println!(
        "cargo:rerun-if-changed={}",
        src.join("CMakeLists.txt").display()
    );

    let mut config = cmake::Config::new(&src);

    if let Ok(sdl_root) = std::env::var("DEP_SDL3_ROOT") {
        config.define("CMAKE_PREFIX_PATH", sdl_root);
    }

    let dst = config
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("SDLMIXER_EXAMPLES", "OFF")
        .define("SDLMIXER_TESTS", "OFF")
        .define("SDLMIXER_VENDORED", "OFF")
        .define("SDLMIXER_STRICT", "OFF")
        .define("SDLMIXER_WAVE", "ON")
        .define("SDLMIXER_VORBIS_STB", "ON")
        .define("SDLMIXER_VORBIS_VORBISFILE", "OFF")
        .define("SDLMIXER_VORBIS_TREMOR", "OFF")
        .define("SDLMIXER_FLAC_DRFLAC", "ON")
        .define("SDLMIXER_FLAC_LIBFLAC", "OFF")
        .define("SDLMIXER_MP3_DRMP3", "ON")
        .define("SDLMIXER_MP3_MPG123", "OFF")
        .define("SDLMIXER_OPUS", "OFF")
        .define("SDLMIXER_WAVPACK", "OFF")
        .define("SDLMIXER_MOD", "OFF")
        .define("SDLMIXER_MIDI", "OFF")
        .define("SDLMIXER_GME", "OFF")
        .define("SDLMIXER_AIFF", "OFF")
        .define("SDLMIXER_VOC", "OFF")
        .define("SDLMIXER_AU", "OFF")
        .build();

    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");

    for dir in ["lib", "lib64"] {
        let path = dst.join(dir);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
    }
    println!(
        "cargo:rustc-link-lib=static={}",
        if msvc { "SDL3_mixer-static" } else { "SDL3_mixer" }
    );
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

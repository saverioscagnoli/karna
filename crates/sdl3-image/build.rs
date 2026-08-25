use std::env;
use std::path::Path;
use std::path::PathBuf;

/// Minimum SDL3_image version this crate is known to work against.
/// Bump this together with the vendored submodule.
const SDL_IMAGE_MIN_VERSION: &str = "3.4.4";

const VENDOR_DIR: &str = "vendor/SDL_image";

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SDL3_IMAGE_LIB_DIR");
    println!("cargo:rerun-if-env-changed=SDL3_IMAGE_INCLUDE_DIR");

    let include_dir = if cfg!(feature = "bundled") {
        build_bundled()
    } else {
        probe_system()
    };

    let (major, minor, micro) = read_header_version(&include_dir);
    println!("cargo:warning=karna-sys: building against SDL3_image {major}.{minor}.{micro}");
    check_version(major, minor, micro);

    // Expose the version to the crate so it can be asserted at runtime.
    println!("cargo:rustc-env=KARNA_SDL_IMAGE_MAJOR={major}");
    println!("cargo:rustc-env=KARNA_SDL_IMAGE_MINOR={minor}");
    println!("cargo:rustc-env=KARNA_SDL_IMAGE_MICRO={micro}");

    // The sdl3 crate's build script exposes where it put SDL3's headers
    // (and, for bundled builds, its cmake install root) via `links`
    // metadata. We need the headers to compile wrapper.h at all, since it
    // pulls in <SDL3/SDL.h>.
    let sdl3_include_dir = PathBuf::from(
        env::var("DEP_SDL3_INCLUDE").expect("sdl3 crate did not export DEP_SDL3_INCLUDE"),
    );

    generate_bindings(&include_dir, &sdl3_include_dir);
}

// ---------------------------------------------------------------------------
// Bundled build
// ---------------------------------------------------------------------------

fn build_bundled() -> PathBuf {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("../../").join(VENDOR_DIR);

    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "SDL_image source not found at {}.\n\
             Run: git submodule update --init --recursive",
            src.display()
        );
    }

    // Rebuild when the submodule pointer moves.
    println!(
        "cargo:rerun-if-changed={}",
        src.join("include/SDL3_image/SDL_image.h").display()
    );

    let mut config = cmake::Config::new(&src);
    config
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("SDLIMAGE_VENDORED", "ON")
        // Default is to dlopen libpng/libwebp/libtiff at runtime. We want a
        // fully self-contained static binary instead, matching the sdl3
        // crate's own bundled build. Each dependency also has its own
        // *_SHARED cache entry seeded from SDLIMAGE_DEPS_SHARED only on
        // first configure, so pin them individually too.
        .define("SDLIMAGE_DEPS_SHARED", "OFF")
        .define("SDLIMAGE_PNG_SHARED", "OFF")
        .define("SDLIMAGE_WEBP_SHARED", "OFF")
        .define("SDLIMAGE_TIF_SHARED", "OFF")
        // AVIF (dav1d + aom) and JXL (libjxl) drag in large third-party
        // encoder/decoder trees we don't need for asset loading. Leave them
        // off; the rest (PNG, JPG, WEBP, TIFF, GIF, BMP, ...) stay on.
        .define("SDLIMAGE_AVIF", "OFF")
        .define("SDLIMAGE_JXL", "OFF")
        .define("SDLIMAGE_SAMPLES", "OFF")
        .define("SDLIMAGE_TESTS", "OFF")
        .define("SDLIMAGE_INSTALL_MAN", "OFF")
        // Rust links PIE on Linux; every object must be relocatable or the
        // final link fails with cryptic relocation errors.
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .profile("Release");

    // Point find_package(SDL3) at the exact SDL3 the sdl3 crate already
    // built, instead of letting cmake search for (or vendor) another copy.
    if let Ok(sdl3_root) = env::var("DEP_SDL3_ROOT") {
        config.define("CMAKE_PREFIX_PATH", sdl3_root);
    }

    let dst = config.build();

    // GNUInstallDirs picks lib64 on some distros (Gentoo, Fedora), lib on others.
    let mut libdir = None;
    for candidate in ["lib", "lib64"] {
        let path = dst.join(candidate);
        if path.exists() {
            println!("cargo:rustc-link-search=native={}", path.display());
            libdir = Some(path);
        }
    }
    let libdir = libdir.expect("cmake produced no lib directory");

    // MSVC installs the static archive as `SDL3_image-static.lib` so it
    // cannot collide with the import library of a shared build; GNU-style
    // builds just produce `libSDL3_image.a`.
    let sdl3_image =
        resolve_lib(&libdir, &["SDL3_image-static", "SDL3_image"]).unwrap_or_else(|| {
            panic!(
                "cmake installed no SDL3_image static library in {}",
                libdir.display()
            )
        });
    println!("cargo:rustc-link-lib=static={sdl3_image}");
    link_transitive_deps(&libdir);

    dst.join("include")
}

/// Static archive naming is not portable: MSVC installs `zlibstatic.lib` and
/// `libpng16_static.lib` where a GNU-style build produces `libz.a` and
/// `libpng16.a`. Given candidate link names, return the first one that
/// actually exists in `libdir`.
fn resolve_lib<'a>(libdir: &Path, candidates: &[&'a str]) -> Option<&'a str> {
    let msvc = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");

    candidates.iter().copied().find(|name| {
        if msvc {
            libdir.join(format!("{name}.lib")).exists()
        } else {
            libdir.join(format!("lib{name}.a")).exists()
        }
    })
}

/// SDL_image's vendored codec libraries (libpng, libwebp, libtiff, zlib, ...)
/// each produce their own static archive that the final binary needs too.
/// pkg-config doesn't know about them (they're private, unexported by
/// sdl3-image.pc's `Requires`), so just link the set this build.rs enables.
fn link_transitive_deps(libdir: &Path) {
    // Dependents before dependencies: webpmux/webpdemux need webp, webp
    // needs sharpyuv, png16 and tiff need zlib. Each entry lists the archive
    // names the codec can install under; a codec that got disabled installs
    // nothing and is skipped.
    const DEPS: &[&[&str]] = &[
        &["libwebpmux", "webpmux"],
        &["libwebpdemux", "webpdemux"],
        &["libwebp", "webp"],
        &["libsharpyuv", "sharpyuv"],
        &["tiff"],
        &["libpng16_static", "png16"],
        &["zlibstatic", "z"],
    ];

    for candidates in DEPS {
        if let Some(lib) = resolve_lib(libdir, candidates) {
            println!("cargo:rustc-link-lib=static={lib}");
        }
    }
}

// ---------------------------------------------------------------------------
// System build
// ---------------------------------------------------------------------------

fn probe_system() -> PathBuf {
    // Manual override, for SDL_image installed outside the default prefix.
    if let (Ok(lib), Ok(inc)) = (
        env::var("SDL3_IMAGE_LIB_DIR"),
        env::var("SDL3_IMAGE_INCLUDE_DIR"),
    ) {
        println!("cargo:rustc-link-search=native={lib}");
        println!("cargo:rustc-link-lib=dylib=SDL3_image");
        return PathBuf::from(inc);
    }

    let sdl3_image = pkg_config::Config::new()
        .atleast_version(SDL_IMAGE_MIN_VERSION)
        .probe("sdl3-image")
        .unwrap_or_else(|e| {
            panic!(
                "SDL3_image >= {SDL_IMAGE_MIN_VERSION} not found via pkg-config: {e}\n\
                 Either install it, or build with --features bundled."
            )
        });

    sdl3_image
        .include_paths
        .first()
        .cloned()
        .expect("pkg-config returned no include path for sdl3-image")
}

// ---------------------------------------------------------------------------
// Version handling
// ---------------------------------------------------------------------------

fn read_header_version(include_dir: &Path) -> (u32, u32, u32) {
    let header = include_dir.join("SDL3_image/SDL_image.h");
    let text = std::fs::read_to_string(&header)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", header.display()));

    let field = |name: &str| -> u32 {
        text.lines()
            .find_map(|line| {
                let line = line.trim();
                let rest = line.strip_prefix("#define")?.trim_start();
                let rest = rest.strip_prefix(name)?;
                // Guard against SDL_IMAGE_MINOR_VERSION matching
                // SDL_IMAGE_MINOR_VERSION_FOO.
                if !rest.starts_with(char::is_whitespace) {
                    return None;
                }
                rest.trim().split_whitespace().next()?.parse().ok()
            })
            .unwrap_or_else(|| panic!("could not parse {name} from {}", header.display()))
    };

    (
        field("SDL_IMAGE_MAJOR_VERSION"),
        field("SDL_IMAGE_MINOR_VERSION"),
        field("SDL_IMAGE_MICRO_VERSION"),
    )
}

/// SDL_image uses parity versioning like SDL3 core: an odd minor OR odd
/// micro means prerelease.
fn check_version(major: u32, minor: u32, micro: u32) {
    if major != 3 {
        panic!("expected SDL3_image 3.x, found {major}.{minor}.{micro}");
    }
    if minor % 2 != 0 || micro % 2 != 0 {
        println!(
            "cargo:warning=SDL3_image {major}.{minor}.{micro} is a PRERELEASE \
             (odd minor or micro). Pin an even.even version for releases."
        );
    }
}

// ---------------------------------------------------------------------------
// Bindgen
// ---------------------------------------------------------------------------

/// Bindings checked into the repo, generated once from the vendored
/// SDL_image headers. Used by default so ordinary builds don't need
/// bindgen/clang-sys at all. Regenerate with `--features codegen` after
/// bumping the vendored SDL_image version, or when building for a platform
/// other than the one these were generated on.
const CHECKED_IN_BINDINGS: &str = "generated/sdl3_image.rs";

fn generate_bindings(include_dir: &Path, sdl3_include_dir: &Path) {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("sdl3_image.rs");
    write_bindings(include_dir, sdl3_include_dir, &out);
}

#[cfg(not(feature = "codegen"))]
fn write_bindings(_include_dir: &Path, _sdl3_include_dir: &Path, out: &Path) {
    println!("cargo:rerun-if-changed={CHECKED_IN_BINDINGS}");
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let checked_in = manifest.join(CHECKED_IN_BINDINGS);
    std::fs::copy(&checked_in, out).unwrap_or_else(|e| {
        panic!(
            "cannot copy checked-in bindings from {}: {e}\n\
             Regenerate with `--features codegen`, or build with that \
             feature enabled directly.",
            checked_in.display()
        )
    });
}

#[cfg(feature = "codegen")]
fn write_bindings(include_dir: &Path, sdl3_include_dir: &Path, out: &Path) {
    bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("-I{}", sdl3_include_dir.display()))
        .use_core()
        .ctypes_prefix("::core::ffi")
        // Keep the surface to SDL_image's own API. The core SDL3 types it
        // takes and returns (SDL_Surface, SDL_Renderer, SDL_IOStream, ...)
        // are blocklisted below and resolved against the `sdl3` crate
        // instead of being redefined here.
        .allowlist_item("IMG_.*")
        .blocklist_item("SDL_.*")
        .raw_line("use sdl3::*;")
        // C enums may legally hold values outside their declared variants.
        // Constructing a Rust enum with an out-of-range discriminant is UB,
        // so use newtypes instead.
        .default_enum_style(bindgen::EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .prepend_enum_name(false)
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_hash(true)
        .generate_comments(true)
        .clang_arg("-fparse-all-comments")
        .layout_tests(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed to generate SDL3_image bindings")
        .write_to_file(out)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", out.display()));
}

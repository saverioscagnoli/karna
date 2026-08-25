use std::env;
use std::path::Path;
use std::path::PathBuf;

/// Minimum SDL3 version this crate is known to work against.
/// Bump this together with the vendored submodule.
const SDL_MIN_VERSION: &str = "3.4.14";

const VENDOR_DIR: &str = "vendor/SDL";

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SDL3_LIB_DIR");
    println!("cargo:rerun-if-env-changed=SDL3_INCLUDE_DIR");

    let include_dir = if cfg!(feature = "bundled") {
        build_bundled()
    } else {
        probe_system()
    };

    // Expose the include dir (and, for bundled builds, the cmake install
    // root) to dependents via `links` metadata, so e.g. sdl3-image can
    // compile/link against the exact same SDL3 this crate built. Visible to
    // direct dependents' build scripts as DEP_SDL3_INCLUDE / DEP_SDL3_ROOT.
    println!("cargo:include={}", include_dir.display());

    let (major, minor, micro) = read_header_version(&include_dir);
    println!("cargo:warning=karna-sys: building against SDL {major}.{minor}.{micro}");
    check_version(major, minor, micro);

    // Expose the version to the crate so it can be asserted at runtime.
    println!("cargo:rustc-env=KARNA_SDL_MAJOR={major}");
    println!("cargo:rustc-env=KARNA_SDL_MINOR={minor}");
    println!("cargo:rustc-env=KARNA_SDL_MICRO={micro}");

    generate_bindings(&include_dir);
    generate_flag_constants(&include_dir);
}

// ---------------------------------------------------------------------------
// Bundled build
// ---------------------------------------------------------------------------

fn build_bundled() -> PathBuf {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("../../").join(VENDOR_DIR);

    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "SDL source not found at {}.\n\
             Run: git submodule update --init --recursive",
            src.display()
        );
    }

    // Rebuild when the submodule pointer moves.
    println!(
        "cargo:rerun-if-changed={}",
        src.join("include/SDL3/SDL_version.h").display()
    );

    let dst = cmake::Config::new(&src)
        .define("SDL_STATIC", "ON")
        .define("SDL_SHARED", "OFF")
        .define("SDL_TESTS", "OFF")
        .define("SDL_TEST_LIBRARY", "OFF")
        .define("SDL_EXAMPLES", "OFF")
        .define("SDL_INSTALL_TESTS", "OFF")
        // Rust links PIE on Linux; every SDL object must be relocatable or
        // the final link fails with cryptic relocation errors.
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .profile("Release")
        .build();

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

    println!("cargo:rustc-link-lib=static=SDL3");
    link_transitive_deps(&libdir);

    // The cmake install prefix, so dependents can point their own cmake
    // builds (e.g. SDL3_image's `find_package(SDL3)`) at it via
    // CMAKE_PREFIX_PATH instead of building/finding a second copy of SDL3.
    println!("cargo:root={}", dst.display());

    dst.join("include")
}

/// SDL needs a pile of platform libraries alongside the static archive.
/// Prefer reading them out of the installed `sdl3.pc`, since that reflects
/// what this particular build actually enabled. Fall back to a hardcoded
/// list if pkg-config isn't available.
fn link_transitive_deps(libdir: &Path) {
    let pc_dir = libdir.join("pkgconfig");

    if pc_dir.join("sdl3.pc").exists() {
        let existing = env::var("PKG_CONFIG_PATH").unwrap_or_default();
        let combined = if existing.is_empty() {
            pc_dir.display().to_string()
        } else {
            format!("{}:{existing}", pc_dir.display())
        };

        unsafe {
            env::set_var("PKG_CONFIG_PATH", combined);
        }

        // We already emitted the SDL3 link directive ourselves.
        let probe = pkg_config::Config::new()
            .statik(true)
            .cargo_metadata(true)
            .probe("sdl3");

        if probe.is_ok() {
            return;
        }
    }

    hardcoded_deps();
}

fn hardcoded_deps() {
    let target = env::var("TARGET").unwrap();

    if target.contains("linux") || target.contains("bsd") {
        // X11, Wayland, ALSA and PulseAudio are dlopen'd at runtime,
        // so they are not needed at link time.
        for lib in ["m", "dl", "pthread", "rt"] {
            println!("cargo:rustc-link-lib=dylib={lib}");
        }
    } else if target.contains("windows") {
        for lib in [
            "user32", "gdi32", "winmm", "imm32", "ole32", "oleaut32", "shell32", "version", "uuid",
            "advapi32", "setupapi", "cfgmgr32",
        ] {
            println!("cargo:rustc-link-lib=dylib={lib}");
        }
    } else if target.contains("darwin") || target.contains("ios") {
        for fw in [
            "CoreVideo",
            "Cocoa",
            "IOKit",
            "ForceFeedback",
            "Carbon",
            "CoreAudio",
            "AudioToolbox",
            "AVFoundation",
            "Foundation",
            "GameController",
            "Metal",
            "QuartzCore",
            "CoreHaptics",
            "UniformTypeIdentifiers",
        ] {
            println!("cargo:rustc-link-lib=framework={fw}");
        }
    }
}

// ---------------------------------------------------------------------------
// System build
// ---------------------------------------------------------------------------

fn probe_system() -> PathBuf {
    // Manual override, for SDL installed outside the default prefix.
    if let (Ok(lib), Ok(inc)) = (env::var("SDL3_LIB_DIR"), env::var("SDL3_INCLUDE_DIR")) {
        println!("cargo:rustc-link-search=native={lib}");
        println!("cargo:rustc-link-lib=dylib=SDL3");
        return PathBuf::from(inc);
    }

    let sdl3 = pkg_config::Config::new()
        .atleast_version(SDL_MIN_VERSION)
        .probe("sdl3")
        .unwrap_or_else(|e| {
            panic!(
                "SDL3 >= {SDL_MIN_VERSION} not found via pkg-config: {e}\n\
                 Either install it (Gentoo: emerge media-libs/libsdl3),\n\
                 or build with --features bundled."
            )
        });

    sdl3.include_paths
        .first()
        .cloned()
        .expect("pkg-config returned no include path for sdl3")
}

// ---------------------------------------------------------------------------
// Version handling
// ---------------------------------------------------------------------------

fn read_header_version(include_dir: &Path) -> (u32, u32, u32) {
    let header = include_dir.join("SDL3/SDL_version.h");
    let text = std::fs::read_to_string(&header)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", header.display()));

    let field = |name: &str| -> u32 {
        text.lines()
            .find_map(|line| {
                let line = line.trim();
                let rest = line.strip_prefix("#define")?.trim_start();
                let rest = rest.strip_prefix(name)?;
                // Guard against SDL_MINOR_VERSION matching SDL_MINOR_VERSION_FOO
                if !rest.starts_with(char::is_whitespace) {
                    return None;
                }
                rest.trim().split_whitespace().next()?.parse().ok()
            })
            .unwrap_or_else(|| panic!("could not parse {name} from {}", header.display()))
    };

    (
        field("SDL_MAJOR_VERSION"),
        field("SDL_MINOR_VERSION"),
        field("SDL_MICRO_VERSION"),
    )
}

/// SDL uses parity versioning: an odd minor OR odd micro means prerelease.
fn check_version(major: u32, minor: u32, micro: u32) {
    if major != 3 {
        panic!("expected SDL 3.x, found {major}.{minor}.{micro}");
    }
    if minor % 2 != 0 || micro % 2 != 0 {
        println!(
            "cargo:warning=SDL {major}.{minor}.{micro} is a PRERELEASE \
             (odd minor or micro). Pin an even.even version for releases."
        );
    }
}

// ---------------------------------------------------------------------------
// Bindgen
// ---------------------------------------------------------------------------

/// Bindings checked into the repo, generated once from the vendored SDL
/// headers. Used by default so ordinary builds don't need bindgen/clang-sys
/// at all. Regenerate with `--features codegen` after bumping the vendored
/// SDL version, or when building for a platform other than the one these
/// were generated on (see the comment on `CHECKED_IN_BINDINGS`).
const CHECKED_IN_BINDINGS: &str = "generated/sdl3.rs";

fn generate_bindings(include_dir: &Path) {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("sdl3.rs");
    write_bindings(include_dir, &out);
}

#[cfg(not(feature = "codegen"))]
fn write_bindings(_include_dir: &Path, out: &Path) {
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
fn write_bindings(include_dir: &Path, out: &Path) {
    bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir.display()))
        .use_core()
        .ctypes_prefix("::core::ffi")
        // Keep the surface to SDL itself; without this you pull in every
        // libc declaration the headers transitively touch.
        .allowlist_item("SDL_.*")
        .allowlist_item("SDLK_.*")
        .allowlist_item("SDL_PROP_.*")
        .blocklist_item("SDL_main")
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
        .expect("bindgen failed to generate SDL3 bindings")
        .write_to_file(out)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", out.display()));
}

/// bindgen drops `#define X SDL_UINT64_C(0x...)` constants because the body
/// is a function-like macro invocation. Recover them by parsing the headers.
fn generate_flag_constants(include_dir: &Path) {
    const HEADERS: &[(&str, &str)] = &[
        ("SDL3/SDL_video.h", "SDL_WINDOW_"),
        ("SDL3/SDL_surface.h", "SDL_SURFACE_"),
    ];

    let mut out = String::from("// @generated by build.rs — do not edit\n\n");

    for (header, prefix) in HEADERS {
        let text = std::fs::read_to_string(include_dir.join(header))
            .unwrap_or_else(|e| panic!("cannot read {header}: {e}"));

        for line in text.lines() {
            let Some(rest) = line.trim().strip_prefix("#define") else {
                continue;
            };
            let rest = rest.trim_start();
            if !rest.starts_with(prefix) {
                continue;
            }

            let mut parts = rest.splitn(2, char::is_whitespace);
            let (Some(name), Some(body)) = (parts.next(), parts.next()) else {
                continue;
            };
            let body = body.trim();

            let (ty, inner) = if let Some(v) = body.strip_prefix("SDL_UINT64_C(") {
                ("u64", v)
            } else if let Some(v) = body.strip_prefix("SDL_UINT32_C(") {
                ("u32", v)
            } else {
                continue;
            };

            let Some(hex) = inner.split(')').next() else {
                continue;
            };
            let hex = hex.trim();

            out.push_str(&format!("pub const {name}: {ty} = {hex};\n"));
        }
    }

    let path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("flags.rs");
    std::fs::write(&path, out).expect("cannot write flags.rs");
}

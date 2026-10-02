#[cfg(feature = "build-from-source")]
use std::path::Path;
#[cfg(feature = "build-from-source")]
use std::path::PathBuf;

/// The library sources, as listed in the makefile's CORE_O, AUX_O and LIB_O.
/// lua.c (the standalone interpreter), ltests.c (internal test hooks) and
/// onelua.c (the amalgamation of everything else) are left out.
#[cfg(feature = "build-from-source")]
const SOURCES: &[&str] = &[
    // core
    "lapi.c", "lcode.c", "lctype.c", "ldebug.c", "ldo.c", "ldump.c", "lfunc.c", "lgc.c",
    "llex.c", "lmem.c", "lobject.c", "lopcodes.c", "lparser.c", "lstate.c", "lstring.c",
    "ltable.c", "ltm.c", "lundump.c", "lvm.c", "lzio.c",
    // auxiliary library
    "lauxlib.c",
    // standard libraries
    "lbaselib.c", "lcorolib.c", "ldblib.c", "linit.c", "liolib.c", "lmathlib.c", "loadlib.c",
    "loslib.c", "lstrlib.c", "ltablib.c", "lutf8lib.c",
];

#[cfg(feature = "build-from-source")]
fn vendored_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/lua")
}

/// Link against a Lua already installed on the system. Distributions disagree
/// on the pkg-config name, so the common spellings are tried in turn.
#[cfg(not(feature = "build-from-source"))]
fn link_system() {
    let mut errors = Vec::new();

    for name in ["lua5.5", "lua-5.5", "lua55", "lua"] {
        match pkg_config::Config::new()
            .atleast_version("5.5")
            .probe(name)
        {
            Ok(_) => return,
            Err(e) => errors.push(format!("{name}: {e}")),
        }
    }

    panic!(
        "could not find a system Lua 5.5 via pkg-config:\n{}\n\
         Either install Lua 5.5 development files, or enable the \
         `build-from-source` feature to build the vendored copy.",
        errors.join("\n")
    );
}

/// Compile the vendored Lua source tree as a static library and link it.
#[cfg(feature = "build-from-source")]
fn build_vendored() {
    let src = vendored_dir();
    if !src.join("lua.h").exists() {
        panic!(
            "vendored Lua not found at {}\n\
             The submodule is not checked out. Run:\n    \
             git submodule update --init --depth 1 vendor/lua",
            src.display()
        );
    }

    let mut build = cc::Build::new();
    build
        .files(SOURCES.iter().map(|f| src.join(f)))
        .include(&src)
        // Lua's own sources are not ours to fix; keep the build log quiet.
        .warnings(false);

    for file in SOURCES {
        println!("cargo:rerun-if-changed={}", src.join(file).display());
    }

    // Mirrors the platform targets in the makefile. Windows needs nothing:
    // luaconf.h turns on LUA_USE_WINDOWS by itself.
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let unix = std::env::var_os("CARGO_CFG_UNIX").is_some();

    match os.as_str() {
        "linux" | "android" => {
            build.define("LUA_USE_LINUX", None);
        }
        "macos" | "ios" => {
            build.define("LUA_USE_MACOSX", None);
        }
        _ if unix => {
            build.define("LUA_USE_POSIX", None);
        }
        _ => {}
    }

    if !build.get_compiler().is_like_msvc() {
        build.flag("-std=c99");
    }

    build.compile("lua");

    // LUA_USE_LINUX turns on dlopen for `require` of C modules.
    if matches!(os.as_str(), "linux" | "android") {
        println!("cargo:rustc-link-lib=dl");
    }
    if unix {
        println!("cargo:rustc-link-lib=m");
    }

    println!("cargo:include={}", src.display());
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    #[cfg(feature = "build-from-source")]
    build_vendored();
    #[cfg(not(feature = "build-from-source"))]
    link_system();
}

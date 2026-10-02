//! Regenerates `src/bindings.rs` from the vendored Lua headers.
//!
//! This is a developer tool, not part of a normal build: the generated file is
//! committed so that users never need libclang installed.
//!
//!     cargo run -p karna-lua-sys --features regenerate --bin regenerate-bindings

use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lua_include = manifest.join("../../vendor/lua");

    assert!(
        lua_include.join("lua.h").exists(),
        "vendored Lua headers not found at {}\n\
         Run: git submodule update --init --depth 1 vendor/lua",
        lua_include.display()
    );

    let out = manifest.join("src/bindings.rs");
    let wrapper = manifest.join("wrapper.h");

    let bindings = bindgen::Builder::default()
        .header(wrapper.to_str().unwrap())
        .clang_arg(format!("-I{}", lua_include.display()))
        // Map C types to core::ffi so the committed file stays correct on
        // targets where c_char signedness or c_long width differ.
        .use_core()
        .ctypes_prefix("core::ffi")
        // Layout tests bake in the host's struct sizes, which would break the
        // committed bindings on other targets.
        .layout_tests(false)
        // Every Lua API that takes or returns one of these constants (status
        // codes, type tags, stack indices) uses `int`, so make them i32
        // instead of bindgen's default u32.
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .allowlist_function("lua_.*|luaL_.*|luaopen_.*")
        .allowlist_type("lua_.*|luaL_.*")
        .allowlist_var("LUA_.*|LUAL_.*")
        // Only reached through luaL_Stream; lib.rs declares it as an opaque
        // type so the platform's struct layout isn't baked in.
        .blocklist_type("FILE|_IO_.*|__.*_t")
        // va_list is a different type on every ABI; nothing on the Rust side
        // can build one anyway.
        .blocklist_function("lua_pushvfstring")
        .blocklist_type("va_list|__builtin_va_list|__va_list_tag")
        .derive_debug(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .generate()
        .expect("bindgen failed");

    bindings.write_to_file(&out).expect("failed to write bindings");

    println!("wrote {}", out.display());
}

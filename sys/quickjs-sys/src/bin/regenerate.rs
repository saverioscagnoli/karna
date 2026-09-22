//! Regenerates `src/bindings.rs` and `src/extern.c` from the vendored QuickJS
//! headers.
//!
//! This is a developer tool, not part of a normal build: the generated files
//! are committed so that users never need libclang installed.
//!
//!     cargo run -p karna-quickjs-sys --features regenerate --bin regenerate-bindings

use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let qjs_include = manifest.join("../../vendor/quickjs");

    assert!(
        qjs_include.join("quickjs.h").exists(),
        "vendored QuickJS headers not found at {}\n\
         Run: git submodule update --init --depth 1 vendor/quickjs",
        qjs_include.display()
    );

    let out = manifest.join("src/bindings.rs");
    let extern_c = manifest.join("src/extern.c");
    let wrapper = manifest.join("wrapper.h");

    let bindings = bindgen::Builder::default()
        .header(wrapper.to_str().unwrap())
        .clang_arg(format!("-I{}", qjs_include.display()))
        // Map C types to core::ffi so the committed file stays correct on
        // targets where c_char signedness or c_long width differ.
        .use_core()
        .ctypes_prefix("core::ffi")
        // Layout tests bake in the host's struct sizes, which would break the
        // committed bindings on other targets.
        .layout_tests(false)
        .allowlist_function("JS_.*")
        .allowlist_type("JS.*")
        .allowlist_var("JS_.*|QJS_.*")
        // Only reached through JS_DumpMemoryUsage; lib.rs declares it as an
        // opaque type so the platform's struct layout isn't baked in.
        .blocklist_type("FILE|_IO_.*|__.*_t")
        // quickjs.h implements part of its API (JS_NewInt32, JS_IsException,
        // JS_ToCString, JS_NewCFunction, ...) as static inline functions,
        // which have no symbol to link against. bindgen writes a C shim that
        // re-exports each one; build.rs compiles it.
        .wrap_static_fns(true)
        .wrap_static_fns_path(&extern_c)
        .derive_debug(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .generate()
        .expect("bindgen failed");

    bindings.write_to_file(&out).expect("failed to write bindings");

    // The shim #includes the wrapper by absolute path; make it relative so
    // the committed file builds on any machine. build.rs puts the crate root
    // on the include path.
    let shim = std::fs::read_to_string(&extern_c).expect("failed to read extern.c");
    let shim = shim.replace(wrapper.to_str().unwrap(), "wrapper.h");
    std::fs::write(&extern_c, shim).expect("failed to write extern.c");

    println!("wrote {}", out.display());
    println!("wrote {}", extern_c.display());
}

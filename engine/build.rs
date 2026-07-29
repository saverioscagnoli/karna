use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out = env::var("OUT_DIR").unwrap();
    let shaders = Path::new(&manifest).join("../shaders");

    for name in ["immediate.vert", "immediate.frag"] {
        let src = shaders.join(name);
        let dst = Path::new(&out).join(format!("{name}.spv"));

        println!("cargo:rerun-if-changed={}", src.display());

        let status = Command::new("glslc")
            .arg(&src)
            .arg("-o")
            .arg(&dst)
            .status()
            .expect("failed to run glslc — install shaderc to build the engine shaders");

        assert!(status.success(), "glslc failed to compile {name}");
    }
}

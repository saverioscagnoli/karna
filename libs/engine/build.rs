use std::env;
use std::path::Path;
use std::process::Command;

fn compile(manifest_dir: &str, out_dir: &str, stage: &str) {
    let src = Path::new(manifest_dir)
        .join("../../shaders")
        .join(format!("immediate.{stage}"));
    let dst = Path::new(out_dir).join(format!("immediate.{stage}.spv"));

    println!("cargo:rerun-if-changed={}", src.display());

    let status = Command::new("glslc")
        .arg(&src)
        .arg("--target-env=vulkan1.0")
        .arg("-o")
        .arg(&dst)
        .status()
        .expect("failed to run glslc (install the Vulkan SDK / shaderc)");

    assert!(
        status.success(),
        "glslc failed to compile {}",
        src.display()
    );
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    compile(&manifest_dir, &out_dir, "vert");
    compile(&manifest_dir, &out_dir, "frag");
}

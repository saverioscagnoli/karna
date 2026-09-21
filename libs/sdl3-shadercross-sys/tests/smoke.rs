use core::ffi::CStr;

use sdl3_shadercross_sys::*;
use sdl3_sys::*;

/// The engine's immediate-mode vertex shader, compiled from GLSL with glslc.
const VERT_SPV: &[u8] = include_bytes!("../../engine/shaders/immediate.vert.spv");

fn spirv_info(code: &[u8], stage: SDL_ShaderCross_ShaderStage) -> SDL_ShaderCross_SPIRV_Info {
    SDL_ShaderCross_SPIRV_Info {
        bytecode: code.as_ptr(),
        bytecode_size: code.len(),
        entrypoint: c"main".as_ptr(),
        shader_stage: stage,
        props: 0,
    }
}

#[test]
fn initializes_and_reports_formats() {
    unsafe {
        assert!(SDL_ShaderCross_Init(), "SDL_ShaderCross_Init failed: {}", get_error());

        let formats = SDL_ShaderCross_GetSPIRVShaderFormats();
        assert!(
            formats & SDL_GPU_SHADERFORMAT_SPIRV != 0,
            "SPIR-V passthrough missing from {formats:#x}"
        );
        assert!(
            formats & SDL_GPU_SHADERFORMAT_MSL != 0,
            "MSL transpilation missing from {formats:#x}"
        );

        SDL_ShaderCross_Quit();
    }
}

#[test]
fn transpiles_spirv_to_msl() {
    unsafe {
        assert!(SDL_ShaderCross_Init(), "SDL_ShaderCross_Init failed: {}", get_error());

        let info = spirv_info(VERT_SPV, SDL_SHADERCROSS_SHADERSTAGE_VERTEX);
        let msl = SDL_ShaderCross_TranspileMSLFromSPIRV(&info);
        assert!(!msl.is_null(), "MSL transpile failed: {}", get_error());

        let source = CStr::from_ptr(msl.cast()).to_str().unwrap();
        assert!(source.contains("#include <metal_stdlib>"), "unexpected MSL:\n{source}");

        SDL_free(msl);
        SDL_ShaderCross_Quit();
    }
}

#[test]
fn transpiles_spirv_to_hlsl() {
    unsafe {
        assert!(SDL_ShaderCross_Init(), "SDL_ShaderCross_Init failed: {}", get_error());

        let info = spirv_info(VERT_SPV, SDL_SHADERCROSS_SHADERSTAGE_VERTEX);
        let hlsl = SDL_ShaderCross_TranspileHLSLFromSPIRV(&info);
        assert!(!hlsl.is_null(), "HLSL transpile failed: {}", get_error());

        let source = CStr::from_ptr(hlsl.cast()).to_str().unwrap();
        assert!(source.contains("main"), "unexpected HLSL:\n{source}");

        SDL_free(hlsl);
        SDL_ShaderCross_Quit();
    }
}

#[test]
fn reflects_graphics_spirv() {
    unsafe {
        let meta = SDL_ShaderCross_ReflectGraphicsSPIRV(VERT_SPV.as_ptr(), VERT_SPV.len(), 0);
        assert!(!meta.is_null(), "reflection failed: {}", get_error());

        assert_eq!((*meta).resource_info.num_uniform_buffers, 1);
        assert_eq!((*meta).num_inputs, 4);

        SDL_free(meta.cast());
    }
}

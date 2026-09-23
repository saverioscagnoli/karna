#![cfg(feature = "shadercross")]

use sdl3::gpu::ShaderFormat;
use sdl3::gpu::ShaderResources;
use sdl3::shadercross::CompileOptions;
use sdl3::shadercross::ShaderCross;
use sdl3::shadercross::ShaderSource;

const VERT: &[u8] = include_bytes!("../../../shaders/immediate.vert.spv");
const FRAG: &[u8] = include_bytes!("../../../shaders/immediate.frag.spv");
const VERT_HLSL: &str = include_str!("../../../shaders/immediate.vert.hlsl");
const FRAG_HLSL: &str = include_str!("../../../shaders/immediate.frag.hlsl");

#[test]
fn shadercross() {
    let sc = ShaderCross::init().unwrap();
    let vert = VERT;
    let frag = FRAG;

    assert!(ShaderCross::init().is_err());
    assert!(
        sc.spirv_formats()
            .contains(ShaderFormat::SPIRV | ShaderFormat::MSL)
    );

    assert_eq!(
        sc.reflect(vert).unwrap(),
        ShaderResources {
            uniform_buffers: 1,
            ..ShaderResources::default()
        }
    );

    let compiled = sc
        .compile(
            ShaderSource::Spirv(frag),
            ShaderFormat::MSL,
            &CompileOptions::fragment(),
        )
        .unwrap();

    assert_eq!(compiled.entrypoint, "main0");
    assert_eq!(compiled.resources.samplers, 1);
    assert!(
        String::from_utf8(compiled.code)
            .unwrap()
            .contains("texture2d_array")
    );

    let all = sc
        .compile_all(
            ShaderSource::Spirv(vert),
            ShaderFormat::SPIRV | ShaderFormat::MSL,
            &CompileOptions::vertex()
                .with_name("immediate.vert")
                .with_debug(true),
        )
        .unwrap();

    assert_eq!(all.len(), 2);
    assert_eq!(all[0].format, ShaderFormat::SPIRV);
    assert_eq!(all[0].code, vert);
    assert_eq!(all[1].format, ShaderFormat::MSL);

    assert!(
        sc.compile(
            ShaderSource::Spirv(vert),
            ShaderFormat::SPIRV | ShaderFormat::MSL,
            &CompileOptions::vertex()
        )
        .is_err()
    );
    assert!(
        sc.compile(
            ShaderSource::Spirv(vert),
            ShaderFormat::METALLIB,
            &CompileOptions::vertex()
        )
        .is_err()
    );

    if cfg!(feature = "dxc") {
        assert!(sc.can_compile_hlsl());

        assert_eq!(
            sc.spirv_from_hlsl(VERT_HLSL, &CompileOptions::vertex())
                .unwrap(),
            VERT,
            "shaders/immediate.vert.spv is stale, run `karna shaders`"
        );
        assert_eq!(
            sc.spirv_from_hlsl(FRAG_HLSL, &CompileOptions::fragment())
                .unwrap(),
            FRAG,
            "shaders/immediate.frag.spv is stale, run `karna shaders`"
        );

        let hlsl = sc
            .compile_all(
                ShaderSource::Hlsl(FRAG_HLSL),
                ShaderFormat::SPIRV | ShaderFormat::MSL,
                &CompileOptions::fragment(),
            )
            .unwrap();

        assert_eq!(hlsl.len(), 2);
        assert_eq!(hlsl[0].resources.samplers, 1);
        assert_eq!(hlsl[1].entrypoint, "main0");
    } else {
        assert!(!sc.can_compile_hlsl());
        assert!(
            sc.compile(
                ShaderSource::Hlsl(FRAG_HLSL),
                ShaderFormat::SPIRV,
                &CompileOptions::fragment()
            )
            .is_err()
        );
    }

    drop(sc);
    drop(ShaderCross::init().unwrap());
}

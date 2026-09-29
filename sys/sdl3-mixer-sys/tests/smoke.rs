use core::ffi::c_void;

use sdl3_mixer_sys::*;
use sdl3_sys::*;

const WAV_SILENCE: &[u8] = &[
    0x52, 0x49, 0x46, 0x46, 0x2C, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6D, 0x74, 0x20,
    0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x44, 0xAC, 0x00, 0x00, 0x88, 0x58, 0x01, 0x00,
    0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

#[test]
fn links_and_reports_version() {
    let v = unsafe { MIX_Version() };
    let major = v / 1_000_000;
    let minor = (v / 1_000) % 1_000;
    assert_eq!(major, 3, "expected SDL3_mixer, got version {v}");
    println!("linked against SDL_mixer {major}.{minor}.{}", v % 1_000);
}

#[test]
fn decodes_wav_from_memory() {
    unsafe {
        assert!(SDL_Init(SDL_INIT_AUDIO), "SDL_Init failed: {}", get_error());
        assert!(MIX_Init(), "MIX_Init failed: {}", get_error());

        let io = SDL_IOFromConstMem(WAV_SILENCE.as_ptr() as *const c_void, WAV_SILENCE.len());
        assert!(!io.is_null(), "SDL_IOFromConstMem failed");

        let decoder = MIX_CreateAudioDecoder_IO(io, true, 0);
        assert!(
            !decoder.is_null(),
            "MIX_CreateAudioDecoder_IO failed: {}",
            get_error()
        );

        let mut spec = SDL_AudioSpec::default();
        assert!(
            MIX_GetAudioDecoderFormat(decoder, &mut spec),
            "MIX_GetAudioDecoderFormat failed: {}",
            get_error()
        );
        assert_eq!((spec.channels, spec.freq), (1, 44_100));

        let mut pcm = [0i16; 4];
        let read = MIX_DecodeAudio(
            decoder,
            pcm.as_mut_ptr() as *mut c_void,
            core::mem::size_of_val(&pcm) as i32,
            &spec,
        );
        assert_eq!(read, 8, "MIX_DecodeAudio failed: {}", get_error());
        assert_eq!(pcm[..4], [0, 0, 0, 0]);

        MIX_DestroyAudioDecoder(decoder);
        MIX_Quit();
        SDL_Quit();
    }
}

use core::ffi::CStr;
use core::ffi::c_int;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::AcqRel;
use core::sync::atomic::Ordering::Release;

use alloc::ffi::CString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use sdl3_mixer_sys::*;
use sdl3_sys::*;

use crate::audio::AudioSpec;
use crate::audio::Sample;

static MIXER_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct MixerGuard(PhantomData<*const ()>);

impl MixerGuard {
    pub fn init() -> Result<Self, SdlError> {
        if MIXER_ACTIVE.swap(true, AcqRel) {
            return Err(SdlError::new(
                "SDL_mixer was initialized more than one time.",
            ));
        }

        if !unsafe { MIX_Init() } {
            MIXER_ACTIVE.store(false, Release);
            return Err(get_error());
        }

        Ok(Self(PhantomData))
    }
}

impl Drop for MixerGuard {
    fn drop(&mut self) {
        unsafe { MIX_Quit() };

        MIXER_ACTIVE.store(false, Release);
    }
}

pub fn linked_version() -> i32 {
    unsafe { MIX_Version() }
}

pub fn decoders() -> Vec<&'static str> {
    let count = unsafe { MIX_GetNumAudioDecoders() }.max(0);

    (0..count)
        .filter_map(|i| {
            let raw = unsafe { MIX_GetAudioDecoder(i) };

            if raw.is_null() {
                return None;
            }

            unsafe { CStr::from_ptr(raw) }.to_str().ok()
        })
        .collect()
}

enum Source {
    Bytes(Arc<[u8]>),
    Path(CString),
}

pub struct Decoder {
    raw: NonNull<MIX_AudioDecoder>,
    source: Source,
    channels: u8,
    freq: u32,
}

impl Decoder {
    pub fn from_bytes<B>(bytes: B) -> Result<Self, SdlError>
    where
        B: Into<Arc<[u8]>>,
    {
        Self::open(Source::Bytes(bytes.into()))
    }

    pub fn from_path<P>(path: P) -> Result<Self, SdlError>
    where
        P: AsRef<str>,
    {
        let path = CString::new(path.as_ref())
            .map_err(|_| SdlError::new("audio path contains an interior nul byte"))?;

        Self::open(Source::Path(path))
    }

    fn open(source: Source) -> Result<Self, SdlError> {
        let raw = Self::create(&source)?;

        let mut spec = SDL_AudioSpec::default();

        if !unsafe { MIX_GetAudioDecoderFormat(raw.as_ptr(), &mut spec) } {
            let err = get_error();
            unsafe { MIX_DestroyAudioDecoder(raw.as_ptr()) };
            return Err(err);
        }

        let native = AudioSpec::from_raw(spec)?;

        Ok(Self {
            raw,
            source,
            channels: native.channels,
            freq: native.freq,
        })
    }

    fn create(source: &Source) -> Result<NonNull<MIX_AudioDecoder>, SdlError> {
        let raw = match source {
            Source::Bytes(bytes) => unsafe {
                let io = SDL_IOFromConstMem(bytes.as_ptr().cast::<c_void>(), bytes.len());

                if io.is_null() {
                    return Err(get_error());
                }

                MIX_CreateAudioDecoder_IO(io, true, 0)
            },

            Source::Path(path) => unsafe { MIX_CreateAudioDecoder(path.as_ptr(), 0) },
        };

        NonNull::new(raw).ok_or_else(get_error)
    }

    pub fn as_ptr(&self) -> *mut MIX_AudioDecoder {
        self.raw.as_ptr()
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn freq(&self) -> u32 {
        self.freq
    }

    pub fn spec<T>(&self) -> AudioSpec
    where
        T: Sample,
    {
        AudioSpec::of::<T>(self.channels, self.freq)
    }

    pub fn set_output(&mut self, channels: u8, freq: u32) {
        self.channels = channels;
        self.freq = freq;
    }

    pub fn decode<T>(&mut self, out: &mut [T]) -> Result<usize, SdlError>
    where
        T: Sample,
    {
        if out.is_empty() {
            return Ok(0);
        }

        let spec = self.spec::<T>().raw();
        let len = mem::size_of_val(out);

        let decoded = unsafe {
            MIX_DecodeAudio(
                self.as_ptr(),
                out.as_mut_ptr().cast::<c_void>(),
                len as c_int,
                &spec,
            )
        };

        if decoded < 0 {
            return Err(get_error());
        }

        Ok(decoded as usize / mem::size_of::<T>())
    }

    pub fn decode_all<T>(&mut self) -> Result<Vec<T>, SdlError>
    where
        T: Sample + Default,
    {
        const CHUNK: usize = 8192;

        let mut out = Vec::new();

        loop {
            let len = out.len();
            out.resize(len + CHUNK, T::default());

            let decoded = self.decode(&mut out[len..])?;
            out.truncate(len + decoded);

            if decoded == 0 {
                break;
            }
        }

        Ok(out)
    }

    pub fn rewind(&mut self) -> Result<(), SdlError> {
        let raw = Self::create(&self.source)?;

        unsafe { MIX_DestroyAudioDecoder(self.raw.as_ptr()) };
        self.raw = raw;

        Ok(())
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe { MIX_DestroyAudioDecoder(self.as_ptr()) };
    }
}

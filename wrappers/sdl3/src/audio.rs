use core::ffi::c_int;
use core::ffi::c_void;
use core::mem;
use core::ptr;
use core::ptr::NonNull;

use sdl3_sys::*;

pub type AudioDeviceId = SDL_AudioDeviceID;

sdl_enum!(AudioFormat: SDL_AudioFormat {
    U8 => SDL_AUDIO_U8,
    S8 => SDL_AUDIO_S8,
    S16 => SDL_AUDIO_S16,
    S32 => SDL_AUDIO_S32,
    F32 => SDL_AUDIO_F32,
});

pub trait Sample: Copy {
    const FORMAT: AudioFormat;
}

impl Sample for u8 {
    const FORMAT: AudioFormat = AudioFormat::U8;
}

impl Sample for i8 {
    const FORMAT: AudioFormat = AudioFormat::S8;
}

impl Sample for i16 {
    const FORMAT: AudioFormat = AudioFormat::S16;
}

impl Sample for i32 {
    const FORMAT: AudioFormat = AudioFormat::S32;
}

impl Sample for f32 {
    const FORMAT: AudioFormat = AudioFormat::F32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioSpec {
    pub format: AudioFormat,
    pub channels: u8,
    pub freq: u32,
}

impl AudioSpec {
    pub const fn new(format: AudioFormat, channels: u8, freq: u32) -> Self {
        Self {
            format,
            channels,
            freq,
        }
    }

    pub const fn of<T>(channels: u8, freq: u32) -> Self
    where
        T: Sample,
    {
        Self::new(T::FORMAT, channels, freq)
    }

    pub const fn raw(self) -> SDL_AudioSpec {
        SDL_AudioSpec {
            format: self.format.raw(),
            channels: self.channels as c_int,
            freq: self.freq as c_int,
        }
    }

    pub fn from_raw(raw: SDL_AudioSpec) -> Result<Self, SdlError> {
        let Some(format) = AudioFormat::from_raw(raw.format) else {
            return Err(SdlError::new("unsupported audio format"));
        };

        Ok(Self {
            format,
            channels: raw.channels as u8,
            freq: raw.freq as u32,
        })
    }

    pub const fn frame_size(self) -> usize {
        let bytes = match self.format {
            AudioFormat::U8 | AudioFormat::S8 => 1,
            AudioFormat::S16 => 2,
            AudioFormat::S32 | AudioFormat::F32 => 4,
        };

        bytes * self.channels as usize
    }

    pub const fn bytes_for(self, duration_ms: u32) -> usize {
        (self.freq as usize * duration_ms as usize / 1000) * self.frame_size()
    }
}

pub struct AudioDevice {
    id: AudioDeviceId,
}

impl AudioDevice {
    pub fn open_default() -> Result<Self, SdlError> {
        Self::open(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, None)
    }

    pub fn open(id: AudioDeviceId, spec: Option<AudioSpec>) -> Result<Self, SdlError> {
        let raw = spec.map(AudioSpec::raw);
        let ptr = raw.as_ref().map_or(ptr::null(), |s| s as *const _);

        let id = unsafe { SDL_OpenAudioDevice(id, ptr) };

        if id == 0 {
            return Err(get_error());
        }

        Ok(Self { id })
    }

    pub fn id(&self) -> AudioDeviceId {
        self.id
    }

    pub fn format(&self) -> Result<(AudioSpec, u32), SdlError> {
        let mut raw = SDL_AudioSpec::default();
        let mut frames: c_int = 0;

        if !unsafe { SDL_GetAudioDeviceFormat(self.id, &mut raw, &mut frames) } {
            return Err(get_error());
        }

        Ok((AudioSpec::from_raw(raw)?, frames.max(0) as u32))
    }

    pub fn gain(&self) -> f32 {
        unsafe { SDL_GetAudioDeviceGain(self.id) }
    }

    pub fn set_gain(&self, gain: f32) -> Result<(), SdlError> {
        if !unsafe { SDL_SetAudioDeviceGain(self.id, gain) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn is_paused(&self) -> bool {
        unsafe { SDL_AudioDevicePaused(self.id) }
    }

    pub fn pause(&self) -> Result<(), SdlError> {
        if !unsafe { SDL_PauseAudioDevice(self.id) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn resume(&self) -> Result<(), SdlError> {
        if !unsafe { SDL_ResumeAudioDevice(self.id) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn bind(&self, stream: &AudioStream) -> Result<(), SdlError> {
        if !unsafe { SDL_BindAudioStream(self.id, stream.as_ptr()) } {
            return Err(get_error());
        }

        Ok(())
    }
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        unsafe { SDL_CloseAudioDevice(self.id) };
    }
}

pub struct AudioStream {
    raw: NonNull<SDL_AudioStream>,
}

impl AudioStream {
    pub fn new(src: AudioSpec) -> Result<Self, SdlError> {
        Self::convert(Some(src), None)
    }

    pub fn convert(src: Option<AudioSpec>, dst: Option<AudioSpec>) -> Result<Self, SdlError> {
        let src_raw = src.map(AudioSpec::raw);
        let dst_raw = dst.map(AudioSpec::raw);

        let raw = unsafe {
            SDL_CreateAudioStream(
                src_raw.as_ref().map_or(ptr::null(), |s| s as *const _),
                dst_raw.as_ref().map_or(ptr::null(), |s| s as *const _),
            )
        };

        NonNull::new(raw)
            .map(|raw| Self { raw })
            .ok_or_else(get_error)
    }

    pub fn as_ptr(&self) -> *mut SDL_AudioStream {
        self.raw.as_ptr()
    }

    pub fn device(&self) -> AudioDeviceId {
        unsafe { SDL_GetAudioStreamDevice(self.as_ptr()) }
    }

    pub fn format(&self) -> Result<(AudioSpec, AudioSpec), SdlError> {
        let mut src = SDL_AudioSpec::default();
        let mut dst = SDL_AudioSpec::default();

        if !unsafe { SDL_GetAudioStreamFormat(self.as_ptr(), &mut src, &mut dst) } {
            return Err(get_error());
        }

        Ok((AudioSpec::from_raw(src)?, AudioSpec::from_raw(dst)?))
    }

    pub fn set_format(
        &self,
        src: Option<AudioSpec>,
        dst: Option<AudioSpec>,
    ) -> Result<(), SdlError> {
        let src_raw = src.map(AudioSpec::raw);
        let dst_raw = dst.map(AudioSpec::raw);

        let ok = unsafe {
            SDL_SetAudioStreamFormat(
                self.as_ptr(),
                src_raw.as_ref().map_or(ptr::null(), |s| s as *const _),
                dst_raw.as_ref().map_or(ptr::null(), |s| s as *const _),
            )
        };

        if !ok {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn put<T>(&self, samples: &[T]) -> Result<(), SdlError>
    where
        T: Sample,
    {
        let len = mem::size_of_val(samples);

        if len == 0 {
            return Ok(());
        }

        let ok = unsafe {
            SDL_PutAudioStreamData(
                self.as_ptr(),
                samples.as_ptr().cast::<c_void>(),
                len as c_int,
            )
        };

        if !ok {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn queued_bytes(&self) -> usize {
        unsafe { SDL_GetAudioStreamQueued(self.as_ptr()) }.max(0) as usize
    }

    pub fn available_bytes(&self) -> usize {
        unsafe { SDL_GetAudioStreamAvailable(self.as_ptr()) }.max(0) as usize
    }

    pub fn gain(&self) -> f32 {
        unsafe { SDL_GetAudioStreamGain(self.as_ptr()) }
    }

    pub fn set_gain(&self, gain: f32) -> Result<(), SdlError> {
        if !unsafe { SDL_SetAudioStreamGain(self.as_ptr(), gain) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn frequency_ratio(&self) -> f32 {
        unsafe { SDL_GetAudioStreamFrequencyRatio(self.as_ptr()) }
    }

    pub fn set_frequency_ratio(&self, ratio: f32) -> Result<(), SdlError> {
        if !unsafe { SDL_SetAudioStreamFrequencyRatio(self.as_ptr(), ratio) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn flush(&self) -> Result<(), SdlError> {
        if !unsafe { SDL_FlushAudioStream(self.as_ptr()) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn clear(&self) -> Result<(), SdlError> {
        if !unsafe { SDL_ClearAudioStream(self.as_ptr()) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn is_device_paused(&self) -> bool {
        unsafe { SDL_AudioStreamDevicePaused(self.as_ptr()) }
    }

    pub fn pause_device(&self) -> Result<(), SdlError> {
        if !unsafe { SDL_PauseAudioStreamDevice(self.as_ptr()) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn resume_device(&self) -> Result<(), SdlError> {
        if !unsafe { SDL_ResumeAudioStreamDevice(self.as_ptr()) } {
            return Err(get_error());
        }

        Ok(())
    }

    pub fn unbind(&self) {
        unsafe { SDL_UnbindAudioStream(self.as_ptr()) };
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        unsafe { SDL_DestroyAudioStream(self.as_ptr()) };
    }
}

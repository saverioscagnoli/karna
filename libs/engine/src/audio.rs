use nostd::alloc::sync::Arc;
use nostd::alloc::vec;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::SlotMap;
use sdl3::audio::AudioDevice;
use sdl3::audio::AudioSpec;
use sdl3::audio::AudioStream;
use sdl3::mixer;
use traccia::warn;

use crate::assets::AssetServer;
use crate::assets::Audio;
use crate::assets::AudioData;
use crate::commands::AudioCommand;
use crate::commands::Outbox;

const STREAM_AHEAD_MS: u32 = 500;
const LOOP_AHEAD_MS: u32 = 100;
const SCRATCH_SAMPLES: usize = 8192;

#[derive(Debug, Clone, Copy)]
pub struct PlayOptions {
    pub looping: bool,
    pub gain: f32,
}

impl Default for PlayOptions {
    fn default() -> Self {
        Self {
            looping: false,
            gain: 1.0,
        }
    }
}

enum Feed {
    Pcm(Arc<[i16]>),
    Decoder(mixer::Decoder),
    Done,
}

pub struct Voice {
    stream: AudioStream,
    spec: AudioSpec,
    feed: Feed,
    looping: bool,
}

impl Voice {
    fn finish(&mut self) {
        let _ = self.stream.flush();
        self.feed = Feed::Done;
    }

    fn pump(&mut self, scratch: &mut [i16]) -> bool {
        match &mut self.feed {
            Feed::Pcm(pcm) => {
                if self.looping && self.stream.queued_bytes() < self.spec.bytes_for(LOOP_AHEAD_MS) {
                    if let Err(e) = self.stream.put(pcm) {
                        warn!("Failed to play audio: {}", e);
                        self.finish();
                    }
                }
            }
            Feed::Decoder(decoder) => {
                let ahead = self.spec.bytes_for(STREAM_AHEAD_MS);
                let mut rewound = false;
                let mut finished = false;

                while self.stream.queued_bytes() < ahead {
                    let n = match decoder.decode(scratch) {
                        Ok(n) => n,
                        Err(e) => {
                            warn!("Failed to play audio: {}", e);
                            finished = true;
                            break;
                        }
                    };

                    if n == 0 {
                        if self.looping && !rewound {
                            rewound = true;

                            if decoder.rewind().is_ok() {
                                continue;
                            }
                        }

                        finished = true;
                        break;
                    }

                    rewound = false;

                    if let Err(e) = self.stream.put(&scratch[..n]) {
                        warn!("Failed to play audio: {}", e);
                        finished = true;
                        break;
                    }
                }

                if finished {
                    self.finish();
                }
            }

            Feed::Done => {}
        }

        match self.feed {
            Feed::Done => self.stream.queued_bytes() > 0,
            Feed::Pcm(_) if !self.looping => self.stream.queued_bytes() > 0,
            _ => true,
        }
    }
}

enum VoiceSlot {
    Reserved,
    Waiting {
        audio: Handle<Audio>,
        options: PlayOptions,
    },
    Playing(Voice),
}

pub struct AudioSystem {
    device: AudioDevice,
    voices: SlotMap<VoiceSlot>,
    dead: Vec<Handle<VoiceSlot>>,
    scratch: Vec<i16>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let Ok(device) = AudioDevice::open_default() else {
            panic!("Failed to open audio device");
        };

        Self {
            device,
            voices: SlotMap::default(),
            dead: Vec::new(),
            scratch: vec![0; SCRATCH_SAMPLES],
        }
    }

    fn reserve(&mut self) -> Handle<Voice> {
        self.voices.insert(VoiceSlot::Reserved).cast()
    }

    fn start(
        device: &AudioDevice,
        scratch: &mut [i16],
        audio: &Audio,
        options: PlayOptions,
    ) -> Option<Voice> {
        let stream = match AudioStream::new(audio.spec) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to play audio: {}", e);
                return None;
            }
        };

        if let Err(e) = device.bind(&stream) {
            warn!("Failed to play audio: {}", e);
            return None;
        }

        let _ = stream.set_gain(options.gain);

        let feed = match &audio.data {
            AudioData::Oneshot(pcm) => {
                if let Err(e) = stream.put(pcm) {
                    warn!("Failed to play audio: {}", e);
                    return None;
                }

                if !options.looping {
                    let _ = stream.flush();
                }

                Feed::Pcm(Arc::clone(pcm))
            }

            AudioData::Streaming(bytes) => match mixer::Decoder::from_bytes(Arc::clone(bytes)) {
                Ok(mut decoder) => {
                    decoder.set_output(audio.spec.channels, audio.spec.freq);
                    Feed::Decoder(decoder)
                }
                Err(e) => {
                    warn!("Failed to play audio: {}", e);
                    return None;
                }
            },
        };

        let mut voice = Voice {
            stream,
            spec: audio.spec,
            feed,
            looping: options.looping,
        };

        voice.pump(scratch);

        Some(voice)
    }

    pub(crate) fn apply(&mut self, command: AudioCommand, assets: &AssetServer) {
        match command {
            AudioCommand::Play {
                audio,
                voice,
                options,
            } => {
                let Some(slot) = self.voices.get_mut(voice.cast()) else {
                    return;
                };

                let Some(source) = assets.try_audio(audio) else {
                    *slot = VoiceSlot::Waiting { audio, options };
                    return;
                };

                match Self::start(&self.device, &mut self.scratch, source, options) {
                    Some(v) => *slot = VoiceSlot::Playing(v),
                    None => {
                        self.voices.remove(voice.cast());
                    }
                }
            }

            AudioCommand::Stop { voice } => {
                self.voices.remove(voice.cast());
            }

            AudioCommand::SetGain { voice, gain } => match self.voices.get_mut(voice.cast()) {
                Some(VoiceSlot::Playing(v)) => {
                    let _ = v.stream.set_gain(gain);
                }
                Some(VoiceSlot::Waiting { options, .. }) => options.gain = gain,
                _ => {}
            },
        }
    }

    pub(crate) fn flush(&mut self, assets: &AssetServer) {
        let Self {
            device,
            voices,
            dead,
            scratch,
        } = self;

        for (handle, slot) in voices.iter_mut() {
            let alive = match slot {
                VoiceSlot::Reserved => false,
                VoiceSlot::Waiting { audio, options } => {
                    let (audio, options) = (*audio, *options);

                    match assets.try_audio(audio) {
                        None => true,
                        Some(source) => match Self::start(device, scratch, source, options) {
                            Some(v) => {
                                *slot = VoiceSlot::Playing(v);
                                true
                            }
                            None => false,
                        },
                    }
                }
                VoiceSlot::Playing(v) => v.pump(scratch),
            };

            if !alive {
                dead.push(handle);
            }
        }

        for handle in dead.drain(..) {
            voices.remove(handle);
        }
    }
}

pub struct AudioHandle<'a> {
    pub(crate) system: &'a mut AudioSystem,
    pub(crate) outbox: &'a mut Outbox<AudioCommand>,
}

impl<'a> AudioHandle<'a> {
    pub fn play(&mut self, audio: Handle<Audio>) -> Handle<Voice> {
        self.play_with(audio, PlayOptions::default())
    }

    pub fn play_with(&mut self, audio: Handle<Audio>, options: PlayOptions) -> Handle<Voice> {
        let voice = self.system.reserve();
        self.outbox.push(AudioCommand::Play {
            audio,
            voice,
            options,
        });
        voice
    }

    pub fn stop(&mut self, voice: Handle<Voice>) {
        self.outbox.push(AudioCommand::Stop { voice });
    }

    pub fn set_gain(&mut self, voice: Handle<Voice>, gain: f32) {
        self.outbox.push(AudioCommand::SetGain { voice, gain });
    }
}

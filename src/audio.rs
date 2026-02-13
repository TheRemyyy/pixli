//! Audio: sound effects and music (rodio).

use std::io::Cursor;
use std::sync::Arc;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

/// Audio source handle (decoded from file or bytes).
#[derive(Clone)]
pub struct AudioSource {
    data: Arc<Vec<u8>>,
}

impl AudioSource {
    /// Load audio from file (WAV, OGG, MP3, FLAC via default rodio/symphonia backends).
    pub fn load(path: &str) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("Failed to load audio: {}", e))?;
        Ok(Self {
            data: Arc::new(data),
        })
    }

    /// Load audio from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            data: Arc::new(bytes),
        }
    }
}

/// Sound handle for controlling playback.
pub struct Sound {
    sink: Option<Arc<Sink>>,
}

impl Sound {
    /// Stop playback.
    pub fn stop(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }

    /// Pause playback.
    pub fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    /// Resume playback.
    pub fn play(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    /// Set volume (0.0–1.0).
    pub fn set_volume(&self, volume: f32) {
        if let Some(sink) = &self.sink {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    /// Returns true if there are still samples to play.
    pub fn is_playing(&self) -> bool {
        self.sink
            .as_ref()
            .map(|sink| !sink.empty())
            .unwrap_or(false)
    }
}

/// Audio system (holds default output stream; play creates sinks).
pub struct Audio {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
}

impl Audio {
    /// Create audio; uses default output device if available.
    pub fn new() -> Self {
        let (output_stream, stream_handle) = match OutputStream::try_default() {
            Ok(pair) => (Some(pair.0), Some(pair.1)),
            Err(e) => {
                log::warn!("Audio: no default output device: {}", e);
                (None, None)
            }
        };
        Self {
            _stream: output_stream,
            stream_handle,
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
        }
    }

    /// Play a sound effect. Returns a handle to control playback; if no device or decode fails, returns a no-op handle.
    pub fn play(&self, source: &AudioSource) -> Sound {
        self.play_volume(source, 1.0)
    }

    /// Play a sound effect with volume (0.0–1.0).
    pub fn play_volume(&self, source: &AudioSource, volume: f32) -> Sound {
        let handle = match self.stream_handle.as_ref() {
            Some(h) => h,
            None => return Sound { sink: None },
        };
        let sink = match Sink::try_new(handle) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Audio: failed to create sink: {}", e);
                return Sound { sink: None };
            }
        };
        let bytes = source.data.as_ref().to_vec();
        let cursor = Cursor::new(bytes);
        let decoder = match Decoder::new(cursor) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("Audio: decode failed: {}", e);
                return Sound { sink: None };
            }
        };
        let vol = (volume * self.sfx_volume * self.master_volume).clamp(0.0, 1.0);
        sink.append(decoder.convert_samples::<f32>().amplify(vol));
        Sound {
            sink: Some(Arc::new(sink)),
        }
    }

    /// Play music (looping).
    pub fn play_music(&self, source: &AudioSource) -> Sound {
        let handle = match self.stream_handle.as_ref() {
            Some(h) => h,
            None => return Sound { sink: None },
        };
        let sink = match Sink::try_new(handle) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Audio: failed to create sink: {}", e);
                return Sound { sink: None };
            }
        };
        let bytes = source.data.as_ref().to_vec();
        let cursor = Cursor::new(bytes);
        let decoder = match Decoder::new(cursor) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("Audio: decode failed: {}", e);
                return Sound { sink: None };
            }
        };
        let vol = (self.music_volume * self.master_volume).clamp(0.0, 1.0);
        sink.append(
            decoder
                .convert_samples::<f32>()
                .amplify(vol)
                .repeat_infinite(),
        );
        Sound {
            sink: Some(Arc::new(sink)),
        }
    }

    /// Set master volume (0.0–1.0).
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume (0.0–1.0).
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Set SFX volume (0.0–1.0).
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Get master volume.
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Get music volume.
    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    /// Get SFX volume.
    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

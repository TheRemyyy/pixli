//! Audio: sound effects and music.

use std::sync::Arc;

/// Audio source handle.
#[derive(Clone)]
pub struct AudioSource {
    #[allow(dead_code)]
    data: Arc<Vec<u8>>,
}

impl AudioSource {
    /// Load audio from file (WAV, OGG, MP3, FLAC).
    pub fn load(path: &str) -> Result<Self, String> {
        let data = std::fs::read(path)
            .map_err(|e| format!("Failed to load audio: {}", e))?;
        Ok(Self { data: Arc::new(data) })
    }

    /// Load audio from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { data: Arc::new(bytes) }
    }
}

/// Sound handle for controlling playback.
pub struct Sound {
    // Internal rodio sink would go here.
    // Placeholder for now.
}

impl Sound {
    /// Stop the sound
    pub fn stop(&self) {
        // rodio sink.stop()
    }

    /// Pause the sound
    pub fn pause(&self) {
        // rodio sink.pause()
    }

    /// Resume the sound
    pub fn play(&self) {
        // rodio sink.play()
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&self, _volume: f32) {
        // rodio sink.set_volume(volume)
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        false // rodio sink.empty()
    }
}

/// Audio system
pub struct Audio {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    // Internal rodio output stream
}

impl Audio {
    pub fn new() -> Self {
        // Initialize rodio
        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
        }
    }

    /// Play a sound effect
    pub fn play(&self, _source: &AudioSource) -> Sound {
        // Create rodio sink, play sound
        Sound {}
    }

    /// Play a sound effect with volume
    pub fn play_volume(&self, _source: &AudioSource, _volume: f32) -> Sound {
        Sound {}
    }

    /// Play music (looping)
    pub fn play_music(&self, _source: &AudioSource) -> Sound {
        Sound {}
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    /// Set SFX volume
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Get master volume
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Get music volume
    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    /// Get SFX volume
    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

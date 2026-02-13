//! Minimal WAV generation for shooter SFX (no external assets).

/// Returns WAV bytes for a short "shoot" click (16-bit mono, 22050 Hz).
pub fn shoot_wav_bytes() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 22050;
    const DURATION_SEC: f32 = 0.06;
    const NUM_SAMPLES: usize = (SAMPLE_RATE as f32 * DURATION_SEC) as usize;
    const DATA_SIZE: u32 = (NUM_SAMPLES * 2) as u32; // 16-bit = 2 bytes per sample
    const FILE_SIZE: u32 = 36 + DATA_SIZE; // 44 - 8 + data

    let mut out = Vec::with_capacity(44 + NUM_SAMPLES * 2);
    // RIFF
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&FILE_SIZE.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    // fmt
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    // data
    out.extend_from_slice(b"data");
    out.extend_from_slice(&DATA_SIZE.to_le_bytes());

    // Short burst with quick decay (loud click so it's clearly audible)
    for i in 0..NUM_SAMPLES {
        let t = i as f32 / SAMPLE_RATE as f32;
        let amp = (1.0 - t / DURATION_SEC).max(0.0);
        let sample = (amp * 18000.0 * (t * 400.0 * std::f32::consts::TAU).sin()) as i16;
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

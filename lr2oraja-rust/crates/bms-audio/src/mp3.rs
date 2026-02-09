/// MP3 decoder using `minimp3`.
use std::path::Path;

use anyhow::{Context, Result};

use crate::pcm::Pcm;

/// Decode an MP3 file from disk.
pub fn decode_file(path: &Path) -> Result<Pcm> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read MP3 file: {}", path.display()))?;
    let mut pcm = decode_bytes(&data)?;
    pcm.strip_trailing_silence();
    Ok(pcm)
}

/// Decode MP3 data from a byte buffer.
///
/// Used by both standalone MP3 files and MP3-in-WAV (format type 85).
pub fn decode_bytes(data: &[u8]) -> Result<Pcm> {
    let mut decoder = minimp3::Decoder::new(std::io::Cursor::new(data));

    let mut all_samples: Vec<f32> = Vec::new();
    let mut channels: u16 = 0;
    let mut sample_rate: u32 = 0;

    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                if channels == 0 {
                    channels = frame.channels as u16;
                    sample_rate = frame.sample_rate as u32;
                }
                for &s in &frame.data {
                    all_samples.push(s as f32 / i16::MAX as f32);
                }
            }
            Err(minimp3::Error::Eof) => break,
            Err(minimp3::Error::SkippedData) => continue,
            Err(e) => return Err(anyhow::anyhow!("MP3 decode error: {e:?}")),
        }
    }

    if channels == 0 || sample_rate == 0 {
        anyhow::bail!("Failed to decode any MP3 frames");
    }

    Ok(Pcm::new(all_samples, channels, sample_rate))
}

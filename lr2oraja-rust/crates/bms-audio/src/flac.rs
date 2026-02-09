/// FLAC decoder using `claxon`.
use std::path::Path;

use anyhow::{Context, Result};

use crate::pcm::Pcm;

/// Decode a FLAC file from disk.
pub fn decode_file(path: &Path) -> Result<Pcm> {
    let mut reader = claxon::FlacReader::open(path)
        .with_context(|| format!("Failed to open FLAC file: {}", path.display()))?;

    let info = reader.streaminfo();
    let channels = info.channels as u16;
    let sample_rate = info.sample_rate;
    let bits_per_sample = info.bits_per_sample;

    let mut all_samples: Vec<f32> = Vec::new();

    // claxon returns interleaved samples via FrameReader
    let mut frame_reader = reader.blocks();
    let mut block_buf = claxon::Block::empty();

    while let Some(block) = frame_reader
        .read_next_or_eof(block_buf.into_buffer())
        .with_context(|| format!("Failed to decode FLAC frame: {}", path.display()))?
    {
        let num_frames = block.len();
        for frame_idx in 0..num_frames {
            for ch in 0..channels as u32 {
                let sample = block.sample(ch, frame_idx);
                all_samples.push(sample_to_f32(sample, bits_per_sample));
            }
        }
        block_buf = block;
    }

    let mut pcm = Pcm::new(all_samples, channels, sample_rate);
    pcm.strip_trailing_silence();
    Ok(pcm)
}

/// Convert a sample to f32 based on bit depth.
///
/// Java FLAC decoder outputs raw bytes with bit-depth info.
/// claxon returns i32 samples regardless of actual bit depth.
fn sample_to_f32(sample: i32, bits_per_sample: u32) -> f32 {
    match bits_per_sample {
        8 => sample as f32 / 128.0,
        16 => sample as f32 / i16::MAX as f32,
        24 => sample as f32 / 8_388_607.0, // 2^23 - 1
        32 => sample as f32 / i32::MAX as f32,
        _ => sample as f32 / ((1i64 << (bits_per_sample - 1)) - 1) as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_to_f32_16bit() {
        assert!((sample_to_f32(0, 16) - 0.0).abs() < 1e-6);
        assert!((sample_to_f32(i16::MAX as i32, 16) - 1.0).abs() < 1e-5);
        assert!((sample_to_f32(i16::MIN as i32, 16) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_sample_to_f32_24bit() {
        assert!((sample_to_f32(0, 24) - 0.0).abs() < 1e-6);
        assert!((sample_to_f32(8_388_607, 24) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sample_to_f32_8bit() {
        assert!((sample_to_f32(0, 8) - 0.0).abs() < 1e-6);
        assert!((sample_to_f32(127, 8) - 0.9921875).abs() < 1e-5);
    }
}

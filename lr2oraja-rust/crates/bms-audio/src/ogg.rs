/// OGG Vorbis decoder using `lewton`.
use std::path::Path;

use anyhow::{Context, Result};

use crate::pcm::Pcm;

/// Decode an OGG Vorbis file from disk.
pub fn decode_file(path: &Path) -> Result<Pcm> {
    let mut reader = lewton::inside_ogg::OggStreamReader::new(
        std::fs::File::open(path)
            .with_context(|| format!("Failed to open OGG file: {}", path.display()))?,
    )
    .with_context(|| format!("Failed to parse OGG stream: {}", path.display()))?;

    let channels = reader.ident_hdr.audio_channels as u16;
    let sample_rate = reader.ident_hdr.audio_sample_rate;

    let mut all_samples: Vec<f32> = Vec::new();

    while let Some(packet) = reader
        .read_dec_packet_itl()
        .with_context(|| format!("Failed to decode OGG packet: {}", path.display()))?
    {
        // lewton returns interleaved i16 samples
        for &s in &packet {
            all_samples.push(s as f32 / i16::MAX as f32);
        }
    }

    let mut pcm = Pcm::new(all_samples, channels, sample_rate);
    pcm.strip_trailing_silence();
    Ok(pcm)
}

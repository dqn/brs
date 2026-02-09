/// WAV file parser.
///
/// Manual RIFF/WAVE chunk parsing, faithfully reproducing Java `PCM.WavInputStream`.
/// Supports format types:
/// - 1 (PCM): 8/16/24/32-bit integer samples
/// - 2 (MS-ADPCM): delegates to `msadpcm::decode()`
/// - 3 (IEEE float): 32-bit float samples
/// - 85 (MP3-in-WAV): delegates to `mp3::decode_bytes()`
use std::io::{self, Read, Seek, SeekFrom};

use anyhow::{Result, bail};

use crate::mp3;
use crate::msadpcm;
use crate::pcm::Pcm;

/// WAVE_FORMAT_EXTENSIBLE format tag (0xFFFE).
///
/// Used by WAV files with >2 channels, >16 bits per sample, or
/// non-standard channel masks. The actual format is in the SubFormat GUID.
const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;

/// WAV format header info extracted from the fmt chunk.
struct WavHeader {
    format_type: u16,
    channels: u16,
    sample_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

/// Read a little-endian u16 from a reader.
fn read_u16_le<R: Read>(reader: &mut R) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

/// Read a little-endian u32 from a reader.
fn read_u32_le<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Read a 4-byte chunk ID.
fn read_chunk_id<R: Read>(reader: &mut R) -> io::Result<[u8; 4]> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Skip `n` bytes in a reader.
fn skip<R: Read>(reader: &mut R, n: usize) -> io::Result<()> {
    let mut remaining = n;
    let mut buf = [0u8; 1024];
    while remaining > 0 {
        let to_read = remaining.min(buf.len());
        reader.read_exact(&mut buf[..to_read])?;
        remaining -= to_read;
    }
    Ok(())
}

/// Decode a WAV file from a reader into a Pcm.
pub fn decode<R: Read + Seek>(reader: &mut R) -> Result<Pcm> {
    // Read RIFF header
    let riff_id = read_chunk_id(reader)?;
    if &riff_id != b"RIFF" {
        bail!("Not a RIFF file");
    }
    let _file_size = read_u32_le(reader)?;

    let wave_id = read_chunk_id(reader)?;
    if &wave_id != b"WAVE" {
        bail!("Not a WAVE file");
    }

    // Find and parse fmt chunk
    let header = seek_and_parse_fmt(reader)?;

    // Find data chunk
    let data_size = seek_to_chunk(reader, b"data")?;

    // Read data
    let mut data = vec![0u8; data_size as usize];
    reader.read_exact(&mut data)?;

    decode_data(&header, &data)
}

/// Seek to a specific chunk by ID, returning the chunk size.
fn seek_to_chunk<R: Read + Seek>(reader: &mut R, target: &[u8; 4]) -> Result<u32> {
    // We need to search from current position for the target chunk.
    // The reader is currently positioned after the WAVE header or after
    // a previous chunk.
    loop {
        let chunk_id = match read_chunk_id(reader) {
            Ok(id) => id,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                bail!("Chunk '{}' not found", String::from_utf8_lossy(target));
            }
            Err(e) => return Err(e.into()),
        };
        let chunk_size = read_u32_le(reader)?;

        if &chunk_id == target {
            return Ok(chunk_size);
        }

        // Skip this chunk (with padding for odd sizes)
        let skip_size = (chunk_size + (chunk_size & 1)) as i64;
        reader.seek(SeekFrom::Current(skip_size))?;
    }
}

/// Seek to fmt chunk and parse it.
fn seek_and_parse_fmt<R: Read + Seek>(reader: &mut R) -> Result<WavHeader> {
    let fmt_size = seek_to_chunk(reader, b"fmt ")?;

    let mut format_type = read_u16_le(reader)?;
    let channels = read_u16_le(reader)?;
    let sample_rate = read_u32_le(reader)?;
    let _byte_rate = read_u32_le(reader)?; // skip avg bytes/sec
    let block_align = read_u16_le(reader)?;
    let bits_per_sample = read_u16_le(reader)?;

    if format_type == WAVE_FORMAT_EXTENSIBLE && fmt_size >= 40 {
        // WAVE_FORMAT_EXTENSIBLE: read cbSize, validBitsPerSample, channelMask, SubFormat
        let _cb_size = read_u16_le(reader)?; // 18: cbSize
        let _valid_bits = read_u16_le(reader)?; // 20: validBitsPerSample
        let _channel_mask = read_u32_le(reader)?; // 22: dwChannelMask
        // SubFormat GUID: first 2 bytes are the actual format type
        let sub_format = read_u16_le(reader)?; // 26: SubFormat[0..2]
        format_type = sub_format;
        // Skip remaining 14 bytes of SubFormat GUID
        skip(reader, 14)?; // 28..40
        // Skip any remaining extra bytes
        if fmt_size > 40 {
            skip(reader, (fmt_size - 40) as usize)?;
        }
    } else if fmt_size > 16 {
        // Skip any extra fmt bytes for non-extensible formats
        skip(reader, (fmt_size - 16) as usize)?;
    }

    Ok(WavHeader {
        format_type,
        channels,
        sample_rate,
        block_align,
        bits_per_sample,
    })
}

/// Decode raw WAV data according to the format type.
fn decode_data(header: &WavHeader, data: &[u8]) -> Result<Pcm> {
    match header.format_type {
        1 => decode_pcm(header, data),
        2 => decode_msadpcm(header, data),
        3 => decode_ieee_float(header, data),
        85 => decode_mp3_in_wav(data),
        _ => bail!("Unsupported WAV format type: {}", header.format_type),
    }
}

/// Decode integer PCM data (format type 1).
fn decode_pcm(header: &WavHeader, data: &[u8]) -> Result<Pcm> {
    let samples = match header.bits_per_sample {
        8 => {
            // 8-bit unsigned → f32
            // Java: (((short)pcm.get()) - 128) / 128.0f
            data.iter().map(|&b| (b as f32 - 128.0) / 128.0).collect()
        }
        16 => {
            // 16-bit signed LE → f32
            let count = data.len() / 2;
            let mut samples = Vec::with_capacity(count);
            for i in 0..count {
                let s = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
                samples.push(s as f32 / i16::MAX as f32);
            }
            samples
        }
        24 => {
            // 24-bit signed LE → f32
            // Java FloatPCM: (((pcm.get(i*3) & 0xff) << 8) | ((pcm.get(i*3+1) & 0xff) << 16) | ((pcm.get(i*3+2) & 0xff) << 24)) / Integer.MAX_VALUE
            let count = data.len() / 3;
            let mut samples = Vec::with_capacity(count);
            for i in 0..count {
                let b0 = data[i * 3] as i32;
                let b1 = data[i * 3 + 1] as i32;
                let b2 = data[i * 3 + 2] as i32;
                // Sign-extend: shift bytes into upper 24 bits of i32
                let val = (b0 << 8) | (b1 << 16) | (b2 << 24);
                samples.push(val as f32 / i32::MAX as f32);
            }
            samples
        }
        32 => {
            // 32-bit signed integer → f32
            // Java ShortPCM: (short)(pcm.getFloat() * Short.MAX_VALUE)
            // We treat as 32-bit int for PCM type 1
            let count = data.len() / 4;
            let mut samples = Vec::with_capacity(count);
            for i in 0..count {
                let s = i32::from_le_bytes([
                    data[i * 4],
                    data[i * 4 + 1],
                    data[i * 4 + 2],
                    data[i * 4 + 3],
                ]);
                samples.push(s as f32 / i32::MAX as f32);
            }
            samples
        }
        bps => bail!("Unsupported PCM bit depth: {bps}"),
    };

    let mut pcm = Pcm::new(samples, header.channels, header.sample_rate);
    pcm.strip_trailing_silence();
    Ok(pcm)
}

/// Decode MS-ADPCM data (format type 2).
fn decode_msadpcm(header: &WavHeader, data: &[u8]) -> Result<Pcm> {
    let decoded_i16 = msadpcm::decode(data, header.channels, header.block_align)?;
    let samples = msadpcm::i16_to_f32(&decoded_i16);
    let mut pcm = Pcm::new(samples, header.channels, header.sample_rate);
    pcm.strip_trailing_silence();
    Ok(pcm)
}

/// Decode IEEE float data (format type 3).
fn decode_ieee_float(header: &WavHeader, data: &[u8]) -> Result<Pcm> {
    match header.bits_per_sample {
        32 => {
            let count = data.len() / 4;
            let mut samples = Vec::with_capacity(count);
            for i in 0..count {
                let f = f32::from_le_bytes([
                    data[i * 4],
                    data[i * 4 + 1],
                    data[i * 4 + 2],
                    data[i * 4 + 3],
                ]);
                samples.push(f);
            }
            let mut pcm = Pcm::new(samples, header.channels, header.sample_rate);
            pcm.strip_trailing_silence();
            Ok(pcm)
        }
        64 => {
            let count = data.len() / 8;
            let mut samples = Vec::with_capacity(count);
            for i in 0..count {
                let d = f64::from_le_bytes([
                    data[i * 8],
                    data[i * 8 + 1],
                    data[i * 8 + 2],
                    data[i * 8 + 3],
                    data[i * 8 + 4],
                    data[i * 8 + 5],
                    data[i * 8 + 6],
                    data[i * 8 + 7],
                ]);
                samples.push(d as f32);
            }
            let mut pcm = Pcm::new(samples, header.channels, header.sample_rate);
            pcm.strip_trailing_silence();
            Ok(pcm)
        }
        bps => bail!("Unsupported IEEE float bit depth: {bps}"),
    }
}

/// Decode MP3-in-WAV (format type 85).
fn decode_mp3_in_wav(data: &[u8]) -> Result<Pcm> {
    let mut pcm = mp3::decode_bytes(data)?;
    pcm.strip_trailing_silence();
    Ok(pcm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Build a minimal WAV file in memory.
    fn build_wav(
        format_type: u16,
        channels: u16,
        sample_rate: u32,
        bits_per_sample: u16,
        data: &[u8],
    ) -> Vec<u8> {
        let fmt_chunk_size: u32 = 16;
        let data_chunk_size = data.len() as u32;
        let file_size = 4 + 8 + fmt_chunk_size + 8 + data_chunk_size;

        let mut buf = Vec::new();
        // RIFF header
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&file_size.to_le_bytes());
        buf.extend_from_slice(b"WAVE");

        // fmt chunk
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&fmt_chunk_size.to_le_bytes());
        buf.extend_from_slice(&format_type.to_le_bytes());
        buf.extend_from_slice(&channels.to_le_bytes());
        buf.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
        buf.extend_from_slice(&byte_rate.to_le_bytes());
        let block_align = channels * bits_per_sample / 8;
        buf.extend_from_slice(&block_align.to_le_bytes());
        buf.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&data_chunk_size.to_le_bytes());
        buf.extend_from_slice(data);

        buf
    }

    #[test]
    fn test_decode_pcm_16bit_mono() {
        // 4 samples: 0, 16383, -16384, 32767
        let mut data = Vec::new();
        for &s in &[0i16, 16383, -16384, 32767] {
            data.extend_from_slice(&s.to_le_bytes());
        }
        let wav = build_wav(1, 1, 44100, 16, &data);
        let pcm = decode(&mut Cursor::new(wav)).unwrap();

        assert_eq!(pcm.channels, 1);
        assert_eq!(pcm.sample_rate, 44100);
        assert_eq!(pcm.num_frames(), 4);
        assert!((pcm.samples[0] - 0.0).abs() < 1e-5);
        assert!((pcm.samples[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_decode_pcm_8bit_mono() {
        // 8-bit unsigned: 128 = silence, 0 = min, 255 = max
        let data = vec![128u8, 0, 255, 64];
        let wav = build_wav(1, 1, 22050, 8, &data);
        let pcm = decode(&mut Cursor::new(wav)).unwrap();

        assert_eq!(pcm.channels, 1);
        assert_eq!(pcm.sample_rate, 22050);
        assert!((pcm.samples[0] - 0.0).abs() < 1e-5); // 128 → 0
        assert!((pcm.samples[1] - (-1.0)).abs() < 0.01); // 0 → -128/128
    }

    #[test]
    fn test_decode_ieee_float() {
        let mut data = Vec::new();
        for &f in &[0.0f32, 0.5, -0.5, 1.0] {
            data.extend_from_slice(&f.to_le_bytes());
        }
        let wav = build_wav(3, 1, 48000, 32, &data);
        let pcm = decode(&mut Cursor::new(wav)).unwrap();

        assert_eq!(pcm.channels, 1);
        assert_eq!(pcm.sample_rate, 48000);
        assert_eq!(pcm.num_frames(), 4);
        assert!((pcm.samples[0] - 0.0).abs() < 1e-6);
        assert!((pcm.samples[1] - 0.5).abs() < 1e-6);
        assert!((pcm.samples[2] - (-0.5)).abs() < 1e-6);
        assert!((pcm.samples[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decode_stereo_16bit() {
        let mut data = Vec::new();
        // 2 frames stereo: (L1, R1), (L2, R2)
        for &s in &[1000i16, -1000, 2000, -2000] {
            data.extend_from_slice(&s.to_le_bytes());
        }
        let wav = build_wav(1, 2, 44100, 16, &data);
        let pcm = decode(&mut Cursor::new(wav)).unwrap();

        assert_eq!(pcm.channels, 2);
        assert_eq!(pcm.num_frames(), 2);
    }

    #[test]
    fn test_trailing_silence_stripped() {
        let mut data = Vec::new();
        // 3 samples: 1000, 0, 0
        for &s in &[1000i16, 0, 0] {
            data.extend_from_slice(&s.to_le_bytes());
        }
        let wav = build_wav(1, 1, 44100, 16, &data);
        let pcm = decode(&mut Cursor::new(wav)).unwrap();
        assert_eq!(pcm.num_frames(), 1); // trailing zeros stripped
    }

    #[test]
    fn test_unsupported_format() {
        let data = vec![0u8; 8];
        let wav = build_wav(99, 1, 44100, 16, &data);
        let result = decode(&mut Cursor::new(wav));
        assert!(result.is_err());
    }

    #[test]
    fn test_not_riff() {
        let data = b"NOT_RIFF_DATA_HERE_NOPE!";
        let result = decode(&mut Cursor::new(data.to_vec()));
        assert!(result.is_err());
    }
}

/// MS-ADPCM decoder.
///
/// Faithful port of Java `MSADPCMDecoder.java`.
/// Decodes Microsoft ADPCM (WAV format ID 0x0002) into signed 16-bit PCM samples.
use anyhow::{Result, bail};

const ADAPTION_TABLE: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230,
];

const INITIALIZATION_COEFF1: [i32; 7] = [64, 128, 0, 48, 60, 115, 98];
const INITIALIZATION_COEFF2: [i32; 7] = [0, -64, 0, 16, 0, -52, -58];

/// Per-channel decoding state.
struct ChannelState {
    adapt_coeff1: i32,
    adapt_coeff2: i32,
    delta: i32,
    sample1: i32,
    sample2: i32,
}

impl ChannelState {
    fn new() -> Self {
        Self {
            adapt_coeff1: 0,
            adapt_coeff2: 0,
            delta: 0,
            sample1: 0,
            sample2: 0,
        }
    }

    /// Expands a 4-bit nibble to a 16-bit PCM sample.
    fn expand_nibble(&mut self, nibble: u8) -> i16 {
        let signed = if nibble >= 8 {
            nibble as i32 - 16
        } else {
            nibble as i32
        };

        let result = (self.sample1 * self.adapt_coeff1) + (self.sample2 * self.adapt_coeff2);
        let predictor = clamp((result >> 6) + (signed * self.delta));

        self.sample2 = self.sample1;
        self.sample1 = predictor as i32;

        self.delta = (ADAPTION_TABLE[nibble as usize] * self.delta) >> 8;
        if self.delta < 16 {
            self.delta = 16;
        }
        if self.delta > i32::MAX / 768 {
            self.delta = i32::MAX / 768;
        }

        predictor
    }
}

fn clamp(value: i32) -> i16 {
    value.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

/// Read a little-endian u16 from a byte slice at the given offset.
fn read_u8(data: &[u8], pos: &mut usize) -> u8 {
    let val = data[*pos];
    *pos += 1;
    val
}

/// Read a little-endian i16 from a byte slice at the given offset.
fn read_i16_le(data: &[u8], pos: &mut usize) -> i16 {
    let val = i16::from_le_bytes([data[*pos], data[*pos + 1]]);
    *pos += 2;
    val
}

/// Decodes MS-ADPCM data into signed i16 PCM samples.
///
/// `data`: raw MS-ADPCM block data.
/// `channels`: number of audio channels.
/// `block_align`: block alignment (bytes per block).
///
/// Returns interleaved i16 PCM samples.
pub fn decode(data: &[u8], channels: u16, block_align: u16) -> Result<Vec<i16>> {
    let ch = channels as usize;
    let block_size = block_align as usize;

    if block_size == 0 || ch == 0 {
        bail!("Invalid MS-ADPCM parameters: channels={ch}, block_align={block_size}");
    }

    // sizeof(header) = 7 per channel
    // samples_per_block = (blockSize - channels * 6) * 2 / channels
    let samples_per_block = (block_size - ch * 6) * 2 / ch;

    if !data.len().is_multiple_of(block_size) {
        bail!(
            "Malformed MS-ADPCM: data length {} is not a multiple of block size {block_size}",
            data.len()
        );
    }

    let block_count = data.len() / block_size;
    let mut output = Vec::with_capacity(block_count * samples_per_block * ch);

    for block_idx in 0..block_count {
        let block_start = block_idx * block_size;
        let block = &data[block_start..block_start + block_size];
        decode_block(block, ch, samples_per_block, &mut output)?;
    }

    Ok(output)
}

fn decode_block(
    block: &[u8],
    ch: usize,
    samples_per_block: usize,
    output: &mut Vec<i16>,
) -> Result<()> {
    let mut pos = 0;

    if ch > 2 {
        // Channels > 2: non-interleaved in block
        let mut channel_samples = vec![vec![0i16; samples_per_block]; ch];
        let mut states: Vec<ChannelState> = (0..ch).map(|_| ChannelState::new()).collect();

        for c in 0..ch {
            let predictor = read_u8(block, &mut pos) as usize;
            if predictor > 6 {
                bail!("Malformed block header: predictor {predictor} > 6");
            }
            states[c].adapt_coeff1 = INITIALIZATION_COEFF1[predictor];
            states[c].adapt_coeff2 = INITIALIZATION_COEFF2[predictor];
            states[c].delta = read_i16_le(block, &mut pos) as i32;
            states[c].sample1 = read_i16_le(block, &mut pos) as i32;
            states[c].sample2 = read_i16_le(block, &mut pos) as i32;

            let mut sample_ptr = 0;
            channel_samples[c][sample_ptr] = states[c].sample2 as i16;
            sample_ptr += 1;
            channel_samples[c][sample_ptr] = states[c].sample1 as i16;
            sample_ptr += 1;

            for _ in 0..((samples_per_block - 2) >> 1) {
                let byte = read_u8(block, &mut pos);
                channel_samples[c][sample_ptr] = states[c].expand_nibble(byte >> 4);
                sample_ptr += 1;
                channel_samples[c][sample_ptr] = states[c].expand_nibble(byte & 0x0f);
                sample_ptr += 1;
            }
        }

        // Interleave
        for i in 0..samples_per_block {
            for samples in channel_samples.iter().take(ch) {
                output.push(samples[i]);
            }
        }
    } else {
        // 1 or 2 channels: interleaved preamble
        let mut states: Vec<ChannelState> = (0..ch).map(|_| ChannelState::new()).collect();

        // Read predictors for all channels
        for state in states.iter_mut().take(ch) {
            let predictor = read_u8(block, &mut pos) as usize;
            if predictor > 6 {
                bail!("Malformed block header: predictor {predictor} > 6");
            }
            state.adapt_coeff1 = INITIALIZATION_COEFF1[predictor];
            state.adapt_coeff2 = INITIALIZATION_COEFF2[predictor];
        }

        // Read deltas
        for state in states.iter_mut().take(ch) {
            state.delta = read_i16_le(block, &mut pos) as i32;
        }

        // Read sample1 for all channels
        for state in states.iter_mut().take(ch) {
            state.sample1 = read_i16_le(block, &mut pos) as i32;
        }

        // Read sample2 for all channels
        for state in states.iter_mut().take(ch) {
            state.sample2 = read_i16_le(block, &mut pos) as i32;
        }

        // Output initial samples (sample2 first, then sample1)
        for state in states.iter().take(ch) {
            output.push(state.sample2 as i16);
        }
        for state in states.iter().take(ch) {
            output.push(state.sample1 as i16);
        }

        // Decode remaining nibbles
        let mut current_ch = 0;
        while pos < block.len() {
            let byte = read_u8(block, &mut pos);

            output.push(states[current_ch].expand_nibble(byte >> 4));
            current_ch = (current_ch + 1) % ch;

            output.push(states[current_ch].expand_nibble(byte & 0x0f));
            current_ch = (current_ch + 1) % ch;
        }
    }

    Ok(())
}

/// Converts decoded i16 samples to f32 in [-1.0, 1.0].
pub fn i16_to_f32(samples: &[i16]) -> Vec<f32> {
    samples
        .iter()
        .map(|&s| s as f32 / i16::MAX as f32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_nibble_positive() {
        let mut state = ChannelState::new();
        state.delta = 16;
        state.sample1 = 0;
        state.sample2 = 0;
        state.adapt_coeff1 = INITIALIZATION_COEFF1[0]; // 64
        state.adapt_coeff2 = INITIALIZATION_COEFF2[0]; // 0

        let result = state.expand_nibble(4);
        assert_eq!(result, 64); // signed=4, result = (0*64 + 0*0)>>6 + 4*16 = 64
    }

    #[test]
    fn test_expand_nibble_negative() {
        let mut state = ChannelState::new();
        state.delta = 16;
        state.sample1 = 0;
        state.sample2 = 0;
        state.adapt_coeff1 = INITIALIZATION_COEFF1[0];
        state.adapt_coeff2 = INITIALIZATION_COEFF2[0];

        let result = state.expand_nibble(12); // signed = 12 - 16 = -4
        assert_eq!(result, -64);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(40000), i16::MAX);
        assert_eq!(clamp(-40000), i16::MIN);
        assert_eq!(clamp(100), 100);
    }

    #[test]
    fn test_decode_mono_block() {
        // Construct a minimal mono MS-ADPCM block
        // block_align = 7 + (N-2)/2 bytes for N samples
        // For simplicity, smallest useful block: 1 channel, block_align = 7 (2 samples only from header)
        // samples_per_block = (7 - 1*6) * 2 / 1 = 2
        let channels = 1u16;
        let block_align = 7u16;

        let mut block = Vec::new();
        // predictor = 0
        block.push(0u8);
        // delta = 16 (LE i16)
        block.extend_from_slice(&16i16.to_le_bytes());
        // sample1 = 100
        block.extend_from_slice(&100i16.to_le_bytes());
        // sample2 = 50
        block.extend_from_slice(&50i16.to_le_bytes());

        let result = decode(&block, channels, block_align).unwrap();
        // samples_per_block = (7 - 6) * 2 / 1 = 2
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], 50); // sample2 first
        assert_eq!(result[1], 100); // sample1 second
    }

    #[test]
    fn test_decode_stereo_block() {
        // Stereo: block_align = 14 (headers only, 2 samples per channel)
        // samples_per_block = (14 - 2*6) * 2 / 2 = 2
        let channels = 2u16;
        let block_align = 14u16;

        let mut block = Vec::new();
        // predictors
        block.push(0u8); // left
        block.push(0u8); // right
        // deltas
        block.extend_from_slice(&16i16.to_le_bytes()); // left
        block.extend_from_slice(&16i16.to_le_bytes()); // right
        // sample1
        block.extend_from_slice(&200i16.to_le_bytes()); // left
        block.extend_from_slice(&(-200i16).to_le_bytes()); // right
        // sample2
        block.extend_from_slice(&100i16.to_le_bytes()); // left
        block.extend_from_slice(&(-100i16).to_le_bytes()); // right

        let result = decode(&block, channels, block_align).unwrap();
        // 2 samples per channel, interleaved = 4 total
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], 100); // L sample2
        assert_eq!(result[1], -100); // R sample2
        assert_eq!(result[2], 200); // L sample1
        assert_eq!(result[3], -200); // R sample1
    }

    #[test]
    fn test_invalid_block_size() {
        let data = vec![0u8; 15]; // Not a multiple of block_align=7
        let result = decode(&data, 1, 7);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_predictor() {
        let mut block = vec![0u8; 7];
        block[0] = 7; // predictor > 6
        let result = decode(&block, 1, 7);
        assert!(result.is_err());
    }

    #[test]
    fn test_i16_to_f32() {
        let samples = vec![0i16, i16::MAX, i16::MIN];
        let result = i16_to_f32(&samples);
        assert!((result[0] - 0.0).abs() < 1e-6);
        assert!((result[1] - 1.0).abs() < 1e-6);
        assert!((result[2] - (-1.0)).abs() < 0.001);
    }
}

/// Unified f32 PCM representation.
///
/// All audio formats are decoded into this common representation
/// with interleaved f32 samples in the range [-1.0, 1.0].
/// PCM audio data with interleaved f32 samples.
#[derive(Debug, Clone)]
pub struct Pcm {
    /// Interleaved sample data [L, R, L, R, ...] in range [-1.0, 1.0].
    pub samples: Vec<f32>,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample rate in Hz.
    pub sample_rate: u32,
}

impl Pcm {
    /// Creates a new Pcm instance.
    pub fn new(samples: Vec<f32>, channels: u16, sample_rate: u32) -> Self {
        Self {
            samples,
            channels,
            sample_rate,
        }
    }

    /// Returns true if the PCM data is valid (non-empty samples, valid params).
    pub fn validate(&self) -> bool {
        !self.samples.is_empty() && self.channels > 0 && self.sample_rate > 0
    }

    /// Returns the duration in microseconds.
    pub fn duration_us(&self) -> i64 {
        if self.channels == 0 || self.sample_rate == 0 {
            return 0;
        }
        let num_frames = self.samples.len() as i64 / self.channels as i64;
        num_frames * 1_000_000 / self.sample_rate as i64
    }

    /// Returns the number of frames (samples per channel).
    pub fn num_frames(&self) -> usize {
        if self.channels == 0 {
            return 0;
        }
        self.samples.len() / self.channels as usize
    }

    /// Changes the sample rate using linear interpolation.
    ///
    /// Faithfully ports Java `ShortPCM.getSample()` / `FloatPCM.getSample()`.
    pub fn change_sample_rate(&self, target_rate: u32) -> Self {
        let samples = self.resample(target_rate);
        Self {
            samples,
            channels: self.channels,
            sample_rate: target_rate,
        }
    }

    /// Changes the number of channels.
    ///
    /// Mono→stereo: duplicates the single channel.
    /// Stereo→mono: takes the first channel (Java behavior).
    /// General: fills all target channels from channel 0 (Java behavior).
    pub fn change_channels(&self, target_channels: u16) -> Self {
        let ch_in = self.channels as usize;
        let ch_out = target_channels as usize;
        let num_frames = self.num_frames();
        let mut out = vec![0.0f32; num_frames * ch_out];

        for i in 0..num_frames {
            for j in 0..ch_out {
                // Java: samples[i * channels + j] = this.sample[i * this.channels]
                // Always reads from channel 0
                out[i * ch_out + j] = self.samples[i * ch_in];
            }
        }

        Self {
            samples: out,
            channels: target_channels,
            sample_rate: self.sample_rate,
        }
    }

    /// Changes the playback speed by resampling.
    ///
    /// rate > 1.0 = faster, rate < 1.0 = slower.
    /// The sample_rate field stays the same; the effective pitch changes.
    pub fn change_frequency(&self, rate: f32) -> Self {
        let effective_rate = (self.sample_rate as f32 / rate) as u32;
        let samples = self.resample(effective_rate);
        Self {
            samples,
            channels: self.channels,
            sample_rate: self.sample_rate,
        }
    }

    /// Slices (trims) the PCM data.
    ///
    /// `start_us`: start time in microseconds.
    /// `duration_us`: duration in microseconds (0 = until end).
    /// Returns None if the result would be empty.
    ///
    /// Strips trailing silence (Java behavior).
    pub fn slice(&self, start_us: i64, duration_us: i64) -> Option<Self> {
        let ch = self.channels as usize;
        let sr = self.sample_rate as i64;
        let total_len = self.samples.len() as i64;

        // Calculate total duration of this PCM in microseconds
        let total_duration_us = total_len * 1_000_000 / (sr * ch as i64);

        let mut dur = duration_us;
        if dur == 0 || start_us + dur > total_duration_us {
            dur = (total_duration_us - start_us).max(0);
        }

        let start_sample = (start_us * sr / 1_000_000) as usize * ch;
        let mut length = (dur * sr / 1_000_000) as usize * ch;

        // Strip trailing silence (Java behavior)
        while length > ch {
            let mut zero = true;
            for i in 0..ch {
                zero &= self.samples[start_sample + length - i - 1] == 0.0;
            }
            if zero {
                length -= ch;
            } else {
                break;
            }
        }

        if length > 0 {
            Some(Self {
                samples: self.samples[start_sample..start_sample + length].to_vec(),
                channels: self.channels,
                sample_rate: self.sample_rate,
            })
        } else {
            None
        }
    }

    /// Core resampling function using linear interpolation.
    ///
    /// Faithfully reproduces Java `ShortPCM.getSample()` / `FloatPCM.getSample()`.
    fn resample(&self, target_rate: u32) -> Vec<f32> {
        let ch = self.channels as usize;
        let src_rate = self.sample_rate as i64;
        let tgt_rate = target_rate as i64;
        let src_frames = self.num_frames() as i64;

        // Java: new float[(int)(((long)this.sample.length / channels) * sample / sampleRate) * channels]
        let dst_frames = (src_frames * tgt_rate / src_rate) as usize;
        let mut out = vec![0.0f32; dst_frames * ch];

        for i in 0..dst_frames as i64 {
            let position = i * src_rate / tgt_rate;
            let modulo = (i * src_rate) % tgt_rate;

            for j in 0..ch {
                if modulo != 0 && ((position + 1) as usize * ch + j) < self.samples.len() {
                    let s1 = self.samples[position as usize * ch + j];
                    let s2 = self.samples[(position + 1) as usize * ch + j];
                    // Linear interpolation: (s1 * (tgt - mod) + s2 * mod) / tgt
                    out[i as usize * ch + j] =
                        (s1 * (tgt_rate - modulo) as f32 + s2 * modulo as f32) / tgt_rate as f32;
                } else {
                    out[i as usize * ch + j] = self.samples[position as usize * ch + j];
                }
            }
        }

        out
    }

    /// Strips trailing silence from the sample buffer.
    ///
    /// Removes frames at the end where all channels are zero.
    /// Matches the Java `PCMLoader.loadPCM()` trailing-silence removal.
    pub fn strip_trailing_silence(&mut self) {
        let ch = self.channels as usize;
        if ch == 0 {
            return;
        }

        let mut len = self.samples.len();
        // Align to frame boundary
        len -= len % ch;

        while len > ch {
            let mut zero = true;
            for i in 0..ch {
                zero &= self.samples[len - i - 1] == 0.0;
            }
            if zero {
                len -= ch;
            } else {
                break;
            }
        }

        self.samples.truncate(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_validate() {
        let pcm = Pcm::new(vec![0.5, -0.5, 0.3, -0.3], 2, 44100);
        assert!(pcm.validate());
        assert_eq!(pcm.num_frames(), 2);
    }

    #[test]
    fn test_empty_invalid() {
        let pcm = Pcm::new(vec![], 2, 44100);
        assert!(!pcm.validate());
    }

    #[test]
    fn test_duration_us() {
        // 44100 Hz, stereo, 44100 frames = 1 second = 1_000_000 us
        let samples = vec![0.0f32; 44100 * 2];
        let pcm = Pcm::new(samples, 2, 44100);
        assert_eq!(pcm.duration_us(), 1_000_000);
    }

    #[test]
    fn test_change_channels_mono_to_stereo() {
        let pcm = Pcm::new(vec![0.5, 0.3, 0.1], 1, 44100);
        let stereo = pcm.change_channels(2);
        assert_eq!(stereo.channels, 2);
        assert_eq!(stereo.samples.len(), 6);
        // Each mono sample duplicated to both channels
        assert_eq!(stereo.samples, vec![0.5, 0.5, 0.3, 0.3, 0.1, 0.1]);
    }

    #[test]
    fn test_change_channels_stereo_to_mono() {
        let pcm = Pcm::new(vec![0.5, -0.5, 0.3, -0.3], 2, 44100);
        let mono = pcm.change_channels(1);
        assert_eq!(mono.channels, 1);
        assert_eq!(mono.samples.len(), 2);
        // Takes channel 0 only (Java behavior)
        assert_eq!(mono.samples, vec![0.5, 0.3]);
    }

    #[test]
    fn test_change_sample_rate_upsample() {
        // 100 frames at 100Hz → 200 frames at 200Hz
        let samples: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let pcm = Pcm::new(samples, 1, 100);
        let upsampled = pcm.change_sample_rate(200);
        assert_eq!(upsampled.sample_rate, 200);
        assert_eq!(upsampled.num_frames(), 200);
        // First sample should be unchanged
        assert!((upsampled.samples[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_change_sample_rate_downsample() {
        // 200 frames at 200Hz → 100 frames at 100Hz
        let samples: Vec<f32> = (0..200).map(|i| i as f32 / 200.0).collect();
        let pcm = Pcm::new(samples, 1, 200);
        let downsampled = pcm.change_sample_rate(100);
        assert_eq!(downsampled.sample_rate, 100);
        assert_eq!(downsampled.num_frames(), 100);
    }

    #[test]
    fn test_change_frequency() {
        let samples = vec![0.0f32; 44100];
        let pcm = Pcm::new(samples, 1, 44100);
        let fast = pcm.change_frequency(2.0);
        // Faster playback = fewer samples, same sample_rate
        assert_eq!(fast.sample_rate, 44100);
        assert!(fast.num_frames() < 44100);
    }

    #[test]
    fn test_slice_basic() {
        // 1 second at 100 Hz mono = 100 samples
        let samples: Vec<f32> = (0..100).map(|i| (i + 1) as f32 / 100.0).collect();
        let pcm = Pcm::new(samples, 1, 100);

        // Slice 200ms starting at 100ms (10 frames starting at frame 10)
        let sliced = pcm.slice(100_000, 200_000).unwrap();
        assert_eq!(sliced.num_frames(), 20);
        assert!((sliced.samples[0] - 0.11).abs() < 1e-6);
    }

    #[test]
    fn test_slice_strips_trailing_silence() {
        // 10 samples: [1.0, 1.0, 0.5, 0.0, 0.0]
        let samples = vec![1.0, 1.0, 0.5, 0.0, 0.0];
        let pcm = Pcm::new(samples, 1, 100);
        let sliced = pcm.slice(0, 0).unwrap();
        // Should strip 2 trailing zeros
        assert_eq!(sliced.num_frames(), 3);
    }

    #[test]
    fn test_slice_all_silence_returns_single_frame() {
        let samples = vec![0.0, 0.0, 0.0, 0.0];
        let pcm = Pcm::new(samples, 2, 44100);
        let sliced = pcm.slice(0, 0);
        // Trailing silence stripping stops at ch (2), so we get 1 frame of zeros
        assert!(sliced.is_some());
        assert_eq!(sliced.unwrap().num_frames(), 1);
    }

    #[test]
    fn test_strip_trailing_silence() {
        let mut pcm = Pcm::new(vec![0.5, -0.5, 0.0, 0.0, 0.0, 0.0], 2, 44100);
        pcm.strip_trailing_silence();
        assert_eq!(pcm.samples.len(), 2); // Only the first frame remains
    }

    #[test]
    fn test_resample_identity() {
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let pcm = Pcm::new(samples.clone(), 1, 44100);
        let resampled = pcm.change_sample_rate(44100);
        assert_eq!(resampled.samples.len(), samples.len());
        for (a, b) in resampled.samples.iter().zip(samples.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_change_sample_rate_stereo() {
        // 4 frames stereo at 100Hz → 8 frames at 200Hz
        let samples = vec![0.1, -0.1, 0.2, -0.2, 0.3, -0.3, 0.4, -0.4];
        let pcm = Pcm::new(samples, 2, 100);
        let upsampled = pcm.change_sample_rate(200);
        assert_eq!(upsampled.channels, 2);
        assert_eq!(upsampled.num_frames(), 8);
        // First frame should be unchanged
        assert!((upsampled.samples[0] - 0.1).abs() < 1e-6);
        assert!((upsampled.samples[1] - (-0.1)).abs() < 1e-6);
    }
}

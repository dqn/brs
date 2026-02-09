/// Offline BMS renderer.
///
/// Renders an entire BMS chart to a PCM buffer by mixing all BGM and key sounds.
/// Ports Java `BMSRenderer.java`.
use std::path::Path;

use anyhow::Result;
use bms_model::BmsModel;

use crate::driver::{AudioDriver, OfflineAudioDriver};
use crate::pcm::Pcm;

/// Result of rendering a BMS chart to audio.
pub struct RenderResult {
    /// Mixed PCM data (interleaved f32).
    pub pcm: Pcm,
    /// Duration in milliseconds.
    pub duration_ms: i64,
}

/// Offline BMS audio renderer.
pub struct BmsRenderer {
    sample_rate: u32,
    channels: u16,
}

impl BmsRenderer {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    /// Render a BMS model to a PCM buffer.
    ///
    /// `max_duration_ms`: maximum render duration in ms (0 = no limit).
    pub fn render(
        &self,
        model: &BmsModel,
        base_path: &Path,
        max_duration_ms: i64,
    ) -> Result<RenderResult> {
        // Load audio files
        let mut driver = OfflineAudioDriver::new(self.sample_rate, self.channels);
        driver.set_model(model, base_path)?;

        // Calculate end time
        let mut end_time_ms = model.last_event_time_ms() as i64;
        if max_duration_ms > 0 && end_time_ms > max_duration_ms {
            end_time_ms = max_duration_ms;
        }

        let total_samples = end_time_ms * self.sample_rate as i64 / 1000;
        let mut mix_buffer = vec![0.0f32; total_samples as usize * self.channels as usize];

        // Mix BG notes
        for bg in &model.bg_notes {
            let time_ms = bg.time_us / 1000;
            if time_ms >= end_time_ms {
                continue;
            }
            if let Some(pcm) =
                driver.get_pcm_for_note(bg.wav_id, bg.micro_starttime, bg.micro_duration)
            {
                mix_pcm(
                    pcm,
                    time_ms,
                    self.sample_rate,
                    self.channels,
                    &mut mix_buffer,
                );
            }
        }

        // Mix playable notes (key sounds)
        for note in &model.notes {
            let time_ms = note.time_us / 1000;
            if time_ms >= end_time_ms {
                continue;
            }
            if let Some(pcm) =
                driver.get_pcm_for_note(note.wav_id, note.micro_starttime, note.micro_duration)
            {
                mix_pcm(
                    pcm,
                    time_ms,
                    self.sample_rate,
                    self.channels,
                    &mut mix_buffer,
                );
            }
        }

        // Apply -6dB headroom and clamp (Java: sample *= 0.5f)
        for sample in &mut mix_buffer {
            *sample *= 0.5;
            *sample = sample.clamp(-1.0, 1.0);
        }

        Ok(RenderResult {
            pcm: Pcm::new(mix_buffer, self.channels, self.sample_rate),
            duration_ms: end_time_ms,
        })
    }
}

impl Default for BmsRenderer {
    fn default() -> Self {
        Self::new(44100, 2)
    }
}

/// Mix PCM data into the output buffer at a given time offset.
fn mix_pcm(pcm: &Pcm, note_time_ms: i64, sample_rate: u32, channels: u16, mix_buffer: &mut [f32]) {
    let start_sample = (note_time_ms * sample_rate as i64 / 1000) as usize * channels as usize;

    for (i, &sample) in pcm.samples.iter().enumerate() {
        let dst = start_sample + i;
        if dst < mix_buffer.len() {
            mix_buffer[dst] += sample;
        }
    }
}

/// Convert f32 mix buffer to i16 samples.
pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped * 32767.0) as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_to_i16() {
        let samples = vec![0.0, 1.0, -1.0, 0.5, -0.5];
        let result = f32_to_i16(&samples);
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 32767);
        assert_eq!(result[2], -32767);
        assert_eq!(result[3], 16383);
        assert_eq!(result[4], -16383);
    }

    #[test]
    fn test_f32_to_i16_clamp() {
        let samples = vec![2.0, -2.0];
        let result = f32_to_i16(&samples);
        assert_eq!(result[0], 32767);
        assert_eq!(result[1], -32767);
    }

    #[test]
    fn test_mix_pcm() {
        let pcm = Pcm::new(vec![0.5, -0.5, 0.3, -0.3], 2, 44100);
        let mut buffer = vec![0.0f32; 10];
        mix_pcm(&pcm, 0, 44100, 2, &mut buffer);
        assert!((buffer[0] - 0.5).abs() < 1e-6);
        assert!((buffer[1] - (-0.5)).abs() < 1e-6);
        assert!((buffer[2] - 0.3).abs() < 1e-6);
        assert!((buffer[3] - (-0.3)).abs() < 1e-6);
    }

    #[test]
    fn test_mix_pcm_additive() {
        let pcm = Pcm::new(vec![0.5], 1, 100);
        let mut buffer = vec![0.3f32; 4];
        mix_pcm(&pcm, 0, 100, 1, &mut buffer);
        assert!((buffer[0] - 0.8).abs() < 1e-6);
        assert!((buffer[1] - 0.3).abs() < 1e-6); // untouched
    }

    #[test]
    fn test_default_renderer() {
        let renderer = BmsRenderer::default();
        assert_eq!(renderer.sample_rate, 44100);
        assert_eq!(renderer.channels, 2);
    }
}

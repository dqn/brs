/// Loudness analyzer using ITU-R BS.1770 (LUFS).
///
/// Ports Java `BMSLoudnessAnalyzer.java`.
/// Uses the `ebur128` crate for integrated loudness measurement.
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::pcm::Pcm;
use crate::renderer::{BmsRenderer, f32_to_i16};

/// Result of a loudness analysis.
#[derive(Debug, Clone)]
pub struct LoudnessResult {
    /// Integrated loudness in LUFS.
    pub loudness_lufs: f64,
    /// SHA256 hash of the analyzed chart.
    pub sha256: String,
}

impl LoudnessResult {
    /// Calculate adjusted volume based on loudness.
    ///
    /// Normalizes to an average of -12 LUFS at 50% volume.
    /// Java: `AVERAGE_LUFS = -12.00`, `gainAdjustment = 10^(-diff/20)`
    pub fn adjusted_volume(&self, base_volume: f32) -> f32 {
        const AVERAGE_LUFS: f64 = -12.0;

        let diff = self.loudness_lufs - AVERAGE_LUFS;
        let gain = 10.0f64.powf(-diff / 20.0);
        let adjusted = 0.5 * gain;

        (adjusted as f32).clamp(0.0, base_volume.max(1.0))
    }
}

/// Analyzes the loudness of PCM audio data.
pub fn analyze_pcm(pcm: &Pcm) -> Result<f64> {
    let channels = pcm.channels as u32;
    let sample_rate = pcm.sample_rate;

    let mut state = ebur128::EbuR128::new(channels, sample_rate, ebur128::Mode::I)
        .context("Failed to create EBU R128 state")?;

    // ebur128 crate expects interleaved i16 samples
    let i16_samples = f32_to_i16(&pcm.samples);
    let frames = i16_samples.len() / channels as usize;
    state
        .add_frames_i16(&i16_samples[..frames * channels as usize])
        .context("Failed to add frames to EBU R128")?;

    let loudness = state
        .loudness_global()
        .context("Failed to get global loudness")?;

    if loudness.is_infinite() && loudness < 0.0 {
        anyhow::bail!("Loudness analysis returned -inf (silent audio)");
    }

    Ok(loudness)
}

/// Loudness analyzer with file-based caching.
pub struct LoudnessAnalyzer {
    cache_dir: PathBuf,
    renderer: BmsRenderer,
}

impl LoudnessAnalyzer {
    /// Create a new analyzer with the given cache directory.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            renderer: BmsRenderer::default(),
        }
    }

    /// Analyze a BMS model's loudness, using cache if available.
    pub fn analyze(&self, model: &bms_model::BmsModel, base_path: &Path) -> Result<LoudnessResult> {
        let sha256 = &model.sha256;

        // Check cache
        if !sha256.is_empty()
            && let Some(cached) = self.read_cache(sha256)
        {
            return Ok(LoudnessResult {
                loudness_lufs: cached,
                sha256: sha256.clone(),
            });
        }

        // Render and analyze
        let result = self.renderer.render(model, base_path, 10 * 60 * 1000)?; // 10 minute limit

        let loudness = analyze_pcm(&result.pcm)?;

        // Cache result
        if !sha256.is_empty() {
            let _ = self.write_cache(sha256, loudness);
        }

        Ok(LoudnessResult {
            loudness_lufs: loudness,
            sha256: sha256.clone(),
        })
    }

    fn cache_path(&self, sha256: &str) -> PathBuf {
        self.cache_dir.join(format!("{sha256}.lufs"))
    }

    fn read_cache(&self, sha256: &str) -> Option<f64> {
        let path = self.cache_path(sha256);
        fs::read_to_string(path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    fn write_cache(&self, sha256: &str, loudness: f64) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        fs::write(self.cache_path(sha256), loudness.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjusted_volume_average() {
        let result = LoudnessResult {
            loudness_lufs: -12.0,
            sha256: String::new(),
        };
        let vol = result.adjusted_volume(1.0);
        assert!((vol - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_adjusted_volume_loud() {
        // Louder than average → lower volume
        let result = LoudnessResult {
            loudness_lufs: -6.0,
            sha256: String::new(),
        };
        let vol = result.adjusted_volume(1.0);
        assert!(vol < 0.5);
    }

    #[test]
    fn test_adjusted_volume_quiet() {
        // Quieter than average → higher volume
        let result = LoudnessResult {
            loudness_lufs: -18.0,
            sha256: String::new(),
        };
        let vol = result.adjusted_volume(1.0);
        assert!(vol > 0.5);
    }

    #[test]
    fn test_analyze_sine_wave() {
        // Generate a 1-second 440Hz sine wave at 44100 Hz, mono
        let sample_rate = 44100u32;
        let duration_samples = sample_rate as usize;
        let samples: Vec<f32> = (0..duration_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.5
            })
            .collect();

        let pcm = Pcm::new(samples, 1, sample_rate);
        let loudness = analyze_pcm(&pcm).unwrap();

        // A sine wave at -6dBFS should be around -9 to -10 LUFS
        assert!(loudness < 0.0, "Loudness should be negative");
        assert!(loudness > -30.0, "Loudness should be reasonable");
    }

    #[test]
    fn test_cache_roundtrip() {
        let dir = std::env::temp_dir().join("bms_loudness_test_cache");
        let _ = fs::remove_dir_all(&dir);
        let analyzer = LoudnessAnalyzer::new(dir.clone());

        analyzer.write_cache("test_hash", -14.5).unwrap();
        let cached = analyzer.read_cache("test_hash");
        assert_eq!(cached, Some(-14.5));

        let _ = fs::remove_dir_all(&dir);
    }
}

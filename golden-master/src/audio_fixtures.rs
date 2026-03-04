// Audio golden master fixture deserialization structures

use serde::Deserialize;

/// Root fixture for audio test cases
#[derive(Debug, Deserialize)]
pub struct AudioFixture {
    pub test_cases: Vec<AudioTestCase>,
}

/// A single audio test case from the Java exporter
#[derive(Debug, Deserialize)]
pub struct AudioTestCase {
    pub name: String,
    pub source_file: String,
    pub operation: String,
    pub channels: Option<u16>,
    pub sample_rate: Option<u32>,
    pub sample_count: usize,
    pub samples_i16: Vec<i16>,
    // Resample fields
    pub source_rate: Option<u32>,
    pub target_rate: Option<u32>,
    // Channel conversion fields
    pub source_channels: Option<u16>,
    pub target_channels: Option<u16>,
}

/// Load audio fixture from JSON file
pub fn load_audio_fixture(path: &std::path::Path) -> anyhow::Result<AudioFixture> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read audio fixture: {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse audio fixture: {}: {}", path.display(), e))
}

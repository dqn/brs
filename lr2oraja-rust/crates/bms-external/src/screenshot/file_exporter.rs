use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;
use tracing::info;

use super::{ScreenshotExporter, ScreenshotScoreInfo};

/// File-based screenshot exporter.
///
/// Saves screenshots to `{output_dir}/YYYYMMDD_HHmmss_{state}.png`.
pub struct FileExporter {
    output_dir: PathBuf,
}

impl FileExporter {
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Generate the output file path for a screenshot.
    pub fn output_path(&self, state_name: &str) -> PathBuf {
        let now = Local::now();
        let filename = format!("{}_{}.png", now.format("%Y%m%d_%H%M%S"), state_name);
        self.output_dir.join(filename)
    }
}

impl ScreenshotExporter for FileExporter {
    fn send(
        &self,
        image_data: &[u8],
        state_name: &str,
        _score_info: Option<&ScreenshotScoreInfo>,
    ) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        let path = self.output_path(state_name);
        fs::write(&path, image_data)?;
        info!("screenshot saved: {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_format() {
        let exporter = FileExporter::new("/tmp/screenshots");
        let path = exporter.output_path("result");
        let filename = path.file_name().unwrap().to_str().unwrap();
        // Format: YYYYMMDD_HHmmss_result.png
        assert!(filename.ends_with("_result.png"));
        assert_eq!(filename.len(), "20260211_123456_result.png".len());
    }

    #[test]
    fn send_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let exporter = FileExporter::new(dir.path());
        let data = b"PNG_DATA";
        exporter.send(data, "play", None).unwrap();

        // Verify file exists
        let entries: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(entries.len(), 1);
        let entry = entries[0].as_ref().unwrap();
        let content = fs::read(entry.path()).unwrap();
        assert_eq!(content, data);
    }

    #[test]
    fn send_creates_output_dir_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("sub").join("dir");
        let exporter = FileExporter::new(&nested);
        exporter.send(b"data", "test", None).unwrap();
        assert!(nested.exists());
    }
}

use std::borrow::Cow;

use anyhow::Result;
use tracing::info;

use super::{ScreenshotExporter, ScreenshotScoreInfo};

/// Screenshot exporter that copies the image to the system clipboard.
pub struct ClipboardExporter;

impl ScreenshotExporter for ClipboardExporter {
    fn send(
        &self,
        image_data: &[u8],
        state_name: &str,
        _score_info: Option<&ScreenshotScoreInfo>,
    ) -> Result<()> {
        let img = image::load_from_memory(image_data)
            .map_err(|e| anyhow::anyhow!("Failed to decode screenshot image: {e}"))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let img_data = arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(rgba.into_raw()),
        };

        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| anyhow::anyhow!("Failed to open clipboard: {e}"))?;
        clipboard
            .set_image(img_data)
            .map_err(|e| anyhow::anyhow!("Failed to copy image to clipboard: {e}"))?;

        info!("screenshot copied to clipboard for {}", state_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_exporter_rejects_invalid_data() {
        let exporter = ClipboardExporter;
        let result = exporter.send(b"not_a_png", "test", None);
        assert!(result.is_err());
    }
}

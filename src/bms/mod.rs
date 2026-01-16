mod bmson;
mod chart;
mod error;
mod loader;
mod timing;

use std::path::Path;

use anyhow::Result;

pub use bmson::*;
pub use chart::*;
pub use error::*;
pub use loader::*;
pub use timing::*;

/// Read a BMS file with automatic encoding detection (UTF-8 or Shift-JIS)
pub fn read_bms_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    // Try UTF-8 first
    if let Ok(content) = std::fs::read_to_string(path) {
        return Ok(content);
    }

    // Fall back to Shift-JIS
    let bytes = std::fs::read(path)?;
    let (content, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&bytes);

    if had_errors {
        anyhow::bail!("Failed to decode file as UTF-8 or Shift-JIS");
    }

    Ok(content.into_owned())
}

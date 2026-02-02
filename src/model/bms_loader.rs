use anyhow::Result;
use bms_rs::bms::prelude::*;
use encoding_rs::{SHIFT_JIS, UTF_8};
use std::path::Path;

/// Load and parse a BMS file from the given path.
/// Automatically detects encoding (UTF-8 or Shift-JIS).
pub fn load_bms(path: &Path) -> Result<Bms> {
    let bytes = std::fs::read(path)?;
    let source = decode_bms_content(&bytes);

    let BmsOutput { bms, warnings: _ } = parse_bms(&source, default_config())
        .map_err(|e| anyhow::anyhow!("BMS parse error: {:?}", e))?;
    Ok(bms)
}

/// Decode BMS file content, trying UTF-8 first then Shift-JIS.
fn decode_bms_content(bytes: &[u8]) -> String {
    let (cow, _, had_errors) = UTF_8.decode(bytes);
    if !had_errors {
        return cow.into_owned();
    }

    let (cow, _, _) = SHIFT_JIS.decode(bytes);
    cow.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_bms() {
        let path = Path::new("bms/bms-002/_take_7N.bms");
        if path.exists() {
            let bms = load_bms(path).expect("Failed to load BMS");
            assert!(bms.music_info.title.is_some());
        }
    }
}

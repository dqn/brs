use anyhow::Result;
use bms_rs::bms::prelude::*;
use bms_rs::bmson::prelude::parse_bmson;
use encoding_rs::{SHIFT_JIS, UTF_8};
use std::path::Path;

/// Source chart format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartFormat {
    Bms,
    Bmson,
}

/// Loaded chart data with detected format.
#[derive(Debug, Clone)]
pub struct LoadedChart {
    pub bms: Bms,
    pub format: ChartFormat,
}

/// Load and parse a BMS file from the given path.
/// Automatically detects encoding (UTF-8 or Shift-JIS).
pub fn load_bms(path: &Path) -> Result<Bms> {
    Ok(load_chart(path)?.bms)
}

/// Load and parse a chart file (BMS or BMSON) from the given path.
pub fn load_chart(path: &Path) -> Result<LoadedChart> {
    let bytes = std::fs::read(path)?;
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if extension == "bmson" {
        let json = String::from_utf8(bytes)
            .map_err(|e| anyhow::anyhow!("BMSON is expected to be UTF-8: {}", e))?;
        let output = parse_bmson(&json);
        let bmson = output
            .bmson
            .ok_or_else(|| anyhow::anyhow!("BMSON parse error: {:?}", output.errors))?;

        let converted = Bms::from_bmson(bmson);
        if !converted.playing_errors.is_empty() {
            return Err(anyhow::anyhow!(
                "BMSON conversion errors: {:?}",
                converted.playing_errors
            ));
        }

        return Ok(LoadedChart {
            bms: converted.bms,
            format: ChartFormat::Bmson,
        });
    }

    let source = decode_bms_content(&bytes);
    let BmsOutput { bms, warnings: _ } = parse_bms(&source, default_config())
        .map_err(|e| anyhow::anyhow!("BMS parse error: {:?}", e))?;
    Ok(LoadedChart {
        bms,
        format: ChartFormat::Bms,
    })
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

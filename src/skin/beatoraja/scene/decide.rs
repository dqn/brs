//! Decide scene skin handling
//!
//! Manages the loading/decide screen between song selection and gameplay.

use super::super::types::BeatorajaSkin;

/// Decide skin configuration extracted from beatoraja skin
#[derive(Debug, Clone, Default)]
pub struct DecideSkinConfig {
    /// Skin width
    pub width: i32,
    /// Skin height
    pub height: i32,
    /// Song info display configuration
    pub song_info: DecideSongInfo,
    /// Loading progress configuration
    pub progress: LoadingProgressConfig,
}

/// Song info display for decide screen
#[derive(Debug, Clone, Default)]
pub struct DecideSongInfo {
    /// Title text element index
    pub title_idx: Option<usize>,
    /// Artist text element index
    pub artist_idx: Option<usize>,
    /// Level number element index
    pub level_idx: Option<usize>,
    /// BPM number element index
    pub bpm_idx: Option<usize>,
}

/// Loading progress display configuration
#[derive(Debug, Clone, Default)]
pub struct LoadingProgressConfig {
    /// Progress bar image ID
    pub progress_bar_idx: Option<usize>,
    /// Progress percentage number index
    pub progress_number_idx: Option<usize>,
}

/// Reference IDs for decide screen elements (beatoraja convention)
pub mod values {
    // Number element value references
    pub const LEVEL: i32 = 6;
    pub const BPM: i32 = 90;
    pub const LOADING_PROGRESS: i32 = 100;

    // Text element string_id references
    pub const TITLE: i32 = 10;
    pub const ARTIST: i32 = 12;
}

impl DecideSkinConfig {
    /// Create from a beatoraja skin
    pub fn from_skin(skin: &BeatorajaSkin) -> Option<Self> {
        // Verify this is a decide skin (type 6)
        if skin.header.skin_type != 6 {
            return None;
        }

        let mut song_info = DecideSongInfo::default();
        let mut progress = LoadingProgressConfig::default();

        // Find number elements
        for (idx, num) in skin.number.iter().enumerate() {
            match num.value {
                values::LEVEL => song_info.level_idx = Some(idx),
                values::BPM => song_info.bpm_idx = Some(idx),
                values::LOADING_PROGRESS => progress.progress_number_idx = Some(idx),
                _ => {}
            }
        }

        // Find text elements
        for (idx, text) in skin.text.iter().enumerate() {
            match text.string_id {
                values::TITLE => song_info.title_idx = Some(idx),
                values::ARTIST => song_info.artist_idx = Some(idx),
                _ => {}
            }
        }

        Some(Self {
            width: skin.header.w,
            height: skin.header.h,
            song_info,
            progress,
        })
    }

    /// Check if this is a valid decide skin
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::beatoraja::types::SkinHeader;

    fn create_test_skin(skin_type: i32) -> BeatorajaSkin {
        BeatorajaSkin {
            header: SkinHeader {
                name: "Test Decide Skin".to_string(),
                author: "Test".to_string(),
                skin_type,
                w: 1920,
                h: 1080,
                path: String::new(),
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_decide_skin_from_skin() {
        let skin = create_test_skin(6); // Decide skin type
        let config = DecideSkinConfig::from_skin(&skin);
        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_decide_skin_wrong_type() {
        let skin = create_test_skin(0); // Play skin type
        let config = DecideSkinConfig::from_skin(&skin);
        assert!(config.is_none());
    }
}

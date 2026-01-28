//! Select scene skin handling
//!
//! Manages song list display, song info, difficulty display, and folder navigation.

use super::super::types::{BeatorajaSkin, NumberElement, TextElement};

/// Select skin configuration extracted from beatoraja skin
#[derive(Debug, Clone, Default)]
pub struct SelectSkinConfig {
    /// Skin width
    pub width: i32,
    /// Skin height
    pub height: i32,
    /// Song list configuration
    pub song_list: SongListConfig,
    /// Song info display configuration
    pub song_info: SongInfoConfig,
    /// Difficulty display configuration
    pub difficulty: DifficultyConfig,
    /// Folder navigation configuration
    pub folder: FolderConfig,
}

/// Song list display configuration
#[derive(Debug, Clone, Default)]
pub struct SongListConfig {
    /// Number of visible items
    pub visible_count: usize,
    /// Selected item index (0-based from top)
    pub selected_index: usize,
    /// Item positions (y offsets)
    pub item_positions: Vec<f32>,
    /// Title text element index
    pub title_text_idx: Option<usize>,
    /// Artist text element index
    pub artist_text_idx: Option<usize>,
    /// Level number element index
    pub level_number_idx: Option<usize>,
}

/// Song info display configuration
#[derive(Debug, Clone, Default)]
pub struct SongInfoConfig {
    /// Title text element index
    pub title_idx: Option<usize>,
    /// Subtitle text element index
    pub subtitle_idx: Option<usize>,
    /// Artist text element index
    pub artist_idx: Option<usize>,
    /// Subartist text element index
    pub subartist_idx: Option<usize>,
    /// Genre text element index
    pub genre_idx: Option<usize>,
    /// BPM number element index
    pub bpm_idx: Option<usize>,
    /// Min BPM number element index
    pub min_bpm_idx: Option<usize>,
    /// Max BPM number element index
    pub max_bpm_idx: Option<usize>,
    /// Total notes number element index
    pub notes_idx: Option<usize>,
    /// Play count number element index
    pub play_count_idx: Option<usize>,
    /// Clear count number element index
    pub clear_count_idx: Option<usize>,
    /// Best score number element index
    pub best_score_idx: Option<usize>,
}

/// Difficulty display configuration
#[derive(Debug, Clone, Default)]
pub struct DifficultyConfig {
    /// Difficulty level number element indices
    pub level_indices: Vec<usize>,
}

/// Folder navigation configuration
#[derive(Debug, Clone, Default)]
pub struct FolderConfig {
    /// Current folder text element index
    pub current_folder_idx: Option<usize>,
}

/// Reference IDs for select screen elements (beatoraja convention)
pub mod values {
    // Number element value references
    pub const LEVEL: i32 = 6;
    pub const BPM: i32 = 90;
    pub const MIN_BPM: i32 = 91;
    pub const MAX_BPM: i32 = 92;
    pub const NOTES: i32 = 74;
    pub const PLAY_COUNT: i32 = 75;
    pub const CLEAR_COUNT: i32 = 76;
    pub const BEST_SCORE: i32 = 71;
    pub const BEST_EXSCORE: i32 = 72;
    pub const BEST_MISSCOUNT: i32 = 73;
    pub const BEST_COMBO: i32 = 77;

    // Text element string_id references
    pub const TITLE: i32 = 10;
    pub const SUBTITLE: i32 = 11;
    pub const ARTIST: i32 = 12;
    pub const SUBARTIST: i32 = 13;
    pub const GENRE: i32 = 14;
    pub const FOLDER_NAME: i32 = 1;
}

impl SelectSkinConfig {
    /// Create from a beatoraja skin
    pub fn from_skin(skin: &BeatorajaSkin) -> Option<Self> {
        // Verify this is a select skin (type 5)
        if skin.header.skin_type != 5 {
            return None;
        }

        let song_info = extract_song_info(&skin.number, &skin.text);
        let difficulty = extract_difficulty_config(&skin.number);

        Some(Self {
            width: skin.header.w,
            height: skin.header.h,
            song_list: SongListConfig::default(),
            song_info,
            difficulty,
            folder: FolderConfig::default(),
        })
    }

    /// Check if this is a valid select skin
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Extract song info configuration from number and text elements
fn extract_song_info(numbers: &[NumberElement], texts: &[TextElement]) -> SongInfoConfig {
    let mut config = SongInfoConfig::default();

    // Find number elements by their value reference
    for (idx, num) in numbers.iter().enumerate() {
        match num.value {
            values::BPM => config.bpm_idx = Some(idx),
            values::MIN_BPM => config.min_bpm_idx = Some(idx),
            values::MAX_BPM => config.max_bpm_idx = Some(idx),
            values::NOTES => config.notes_idx = Some(idx),
            values::PLAY_COUNT => config.play_count_idx = Some(idx),
            values::CLEAR_COUNT => config.clear_count_idx = Some(idx),
            values::BEST_SCORE | values::BEST_EXSCORE => config.best_score_idx = Some(idx),
            _ => {}
        }
    }

    // Find text elements by their string_id reference
    for (idx, text) in texts.iter().enumerate() {
        match text.string_id {
            values::TITLE => config.title_idx = Some(idx),
            values::SUBTITLE => config.subtitle_idx = Some(idx),
            values::ARTIST => config.artist_idx = Some(idx),
            values::SUBARTIST => config.subartist_idx = Some(idx),
            values::GENRE => config.genre_idx = Some(idx),
            _ => {}
        }
    }

    config
}

/// Extract difficulty configuration from number elements
fn extract_difficulty_config(numbers: &[NumberElement]) -> DifficultyConfig {
    let mut config = DifficultyConfig::default();

    // Find level number elements
    for (idx, num) in numbers.iter().enumerate() {
        if num.value == values::LEVEL {
            config.level_indices.push(idx);
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::beatoraja::types::SkinHeader;

    fn create_test_skin(skin_type: i32) -> BeatorajaSkin {
        BeatorajaSkin {
            header: SkinHeader {
                name: "Test Select Skin".to_string(),
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
    fn test_select_skin_from_skin() {
        let skin = create_test_skin(5); // Select skin type
        let config = SelectSkinConfig::from_skin(&skin);
        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_select_skin_wrong_type() {
        let skin = create_test_skin(0); // Play skin type
        let config = SelectSkinConfig::from_skin(&skin);
        assert!(config.is_none());
    }

    #[test]
    fn test_extract_song_info() {
        use crate::skin::beatoraja::types::ElementBase;

        let numbers = vec![
            NumberElement {
                base: ElementBase::default(),
                id: 100,
                value: values::BPM,
                digit: 3,
                padding: 0,
                align: 0,
                dst: vec![],
            },
            NumberElement {
                base: ElementBase::default(),
                id: 101,
                value: values::NOTES,
                digit: 4,
                padding: 0,
                align: 0,
                dst: vec![],
            },
        ];
        let texts = vec![TextElement {
            base: ElementBase::default(),
            font: 0,
            string_id: values::TITLE,
            text: String::new(),
            align: 0,
            dst: vec![],
        }];

        let config = extract_song_info(&numbers, &texts);
        assert_eq!(config.bpm_idx, Some(0));
        assert_eq!(config.notes_idx, Some(1));
        assert_eq!(config.title_idx, Some(0));
    }
}

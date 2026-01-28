//! Result scene skin handling
//!
//! Manages score display, judge statistics, clear lamp, and grade display.

use super::super::types::BeatorajaSkin;

/// Result skin configuration extracted from beatoraja skin
#[derive(Debug, Clone, Default)]
pub struct ResultSkinConfig {
    /// Skin width
    pub width: i32,
    /// Skin height
    pub height: i32,
    /// Song info display configuration
    pub song_info: ResultSongInfo,
    /// Score display configuration
    pub score: ScoreDisplayConfig,
    /// Judge statistics configuration
    pub judge_stats: JudgeStatsConfig,
    /// Clear result configuration
    pub clear_result: ClearResultConfig,
    /// IR ranking configuration
    pub ranking: RankingConfig,
}

/// Song info display for result screen
#[derive(Debug, Clone, Default)]
pub struct ResultSongInfo {
    /// Title text element index
    pub title_idx: Option<usize>,
    /// Artist text element index
    pub artist_idx: Option<usize>,
    /// Level number element index
    pub level_idx: Option<usize>,
}

/// Score display configuration
#[derive(Debug, Clone, Default)]
pub struct ScoreDisplayConfig {
    /// EX Score number element index
    pub ex_score_idx: Option<usize>,
    /// Score rate number element index
    pub score_rate_idx: Option<usize>,
    /// Max combo number element index
    pub max_combo_idx: Option<usize>,
    /// Best score difference number element index
    pub score_diff_idx: Option<usize>,
    /// Target score difference number element index
    pub target_diff_idx: Option<usize>,
}

/// Judge statistics configuration
#[derive(Debug, Clone, Default)]
pub struct JudgeStatsConfig {
    /// PGREAT count number element index
    pub pgreat_idx: Option<usize>,
    /// GREAT count number element index
    pub great_idx: Option<usize>,
    /// GOOD count number element index
    pub good_idx: Option<usize>,
    /// BAD count number element index
    pub bad_idx: Option<usize>,
    /// POOR count number element index
    pub poor_idx: Option<usize>,
    /// Miss count number element index
    pub miss_idx: Option<usize>,
    /// Fast count number element index
    pub fast_idx: Option<usize>,
    /// Slow count number element index
    pub slow_idx: Option<usize>,
}

/// Clear result display configuration
#[derive(Debug, Clone, Default)]
pub struct ClearResultConfig {
    /// Clear lamp image indices by clear type
    pub clear_lamp_indices: Vec<usize>,
    /// DJ level/grade image indices
    pub grade_indices: Vec<usize>,
    /// New record image index
    pub new_record_idx: Option<usize>,
}

/// IR ranking display configuration
#[derive(Debug, Clone, Default)]
pub struct RankingConfig {
    /// Current rank number element index
    pub rank_idx: Option<usize>,
    /// Total players number element index
    pub total_players_idx: Option<usize>,
}

/// Reference IDs for result screen elements (beatoraja convention)
pub mod values {
    // Number element value references
    pub const EX_SCORE: i32 = 71;
    pub const SCORE_RATE: i32 = 102;
    pub const MAX_COMBO: i32 = 77;
    pub const MISS_COUNT: i32 = 73;
    pub const LEVEL: i32 = 6;

    // Judge count references
    pub const PGREAT: i32 = 110;
    pub const GREAT: i32 = 111;
    pub const GOOD: i32 = 112;
    pub const BAD: i32 = 113;
    pub const POOR: i32 = 114;
    pub const COMBO_BREAK: i32 = 115;
    pub const FAST: i32 = 120;
    pub const SLOW: i32 = 121;

    // IR ranking references
    pub const IR_RANK: i32 = 200;
    pub const IR_TOTAL: i32 = 201;

    // Text element string_id references
    pub const TITLE: i32 = 10;
    pub const ARTIST: i32 = 12;
}

impl ResultSkinConfig {
    /// Create from a beatoraja skin
    pub fn from_skin(skin: &BeatorajaSkin) -> Option<Self> {
        // Verify this is a result skin (type 7)
        if skin.header.skin_type != 7 {
            return None;
        }

        let mut song_info = ResultSongInfo::default();
        let mut score = ScoreDisplayConfig::default();
        let mut judge_stats = JudgeStatsConfig::default();
        let mut ranking = RankingConfig::default();

        // Find number elements
        for (idx, num) in skin.number.iter().enumerate() {
            match num.value {
                values::LEVEL => song_info.level_idx = Some(idx),
                values::EX_SCORE => score.ex_score_idx = Some(idx),
                values::SCORE_RATE => score.score_rate_idx = Some(idx),
                values::MAX_COMBO => score.max_combo_idx = Some(idx),
                values::PGREAT => judge_stats.pgreat_idx = Some(idx),
                values::GREAT => judge_stats.great_idx = Some(idx),
                values::GOOD => judge_stats.good_idx = Some(idx),
                values::BAD => judge_stats.bad_idx = Some(idx),
                values::POOR => judge_stats.poor_idx = Some(idx),
                values::COMBO_BREAK => judge_stats.miss_idx = Some(idx),
                values::FAST => judge_stats.fast_idx = Some(idx),
                values::SLOW => judge_stats.slow_idx = Some(idx),
                values::IR_RANK => ranking.rank_idx = Some(idx),
                values::IR_TOTAL => ranking.total_players_idx = Some(idx),
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
            score,
            judge_stats,
            clear_result: ClearResultConfig::default(),
            ranking,
        })
    }

    /// Check if this is a valid result skin
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
                name: "Test Result Skin".to_string(),
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
    fn test_result_skin_from_skin() {
        let skin = create_test_skin(7); // Result skin type
        let config = ResultSkinConfig::from_skin(&skin);
        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_result_skin_wrong_type() {
        let skin = create_test_skin(0); // Play skin type
        let config = ResultSkinConfig::from_skin(&skin);
        assert!(config.is_none());
    }
}

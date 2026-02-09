// PlayerResource — shared data container passed between states.
//
// Contains the currently loaded BMS model, score data, and play settings.
// Phase 11 A+B uses a minimal set of fields; more will be added as needed.

use bms_config::PlayerConfig;
use bms_model::{BmsModel, PlayMode};
use bms_rule::ScoreData;

/// Data shared across game states during a play session.
#[derive(Debug, Clone)]
pub struct PlayerResource {
    /// Currently loaded BMS chart (None if nothing is loaded).
    pub bms_model: Option<BmsModel>,
    /// Score data for the current play session.
    #[allow(dead_code)]
    pub score_data: ScoreData,
    /// Active play mode.
    pub play_mode: PlayMode,
    /// Player configuration snapshot.
    #[allow(dead_code)]
    pub player_config: PlayerConfig,
    /// Original gauge option (saved at Decide, may be modified during Play).
    pub org_gauge_option: i32,
}

impl Default for PlayerResource {
    fn default() -> Self {
        Self {
            bms_model: None,
            score_data: ScoreData::default(),
            play_mode: PlayMode::Beat7K,
            player_config: PlayerConfig::default(),
            org_gauge_option: 0,
        }
    }
}

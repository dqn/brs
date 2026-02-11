// PlayerResource — shared data container passed between states.
//
// Contains the currently loaded BMS model, score data, and play settings.
// Used to pass data between Decide -> Play -> Result states.

use std::path::PathBuf;

use bms_config::PlayerConfig;
use bms_model::{BmsModel, PlayMode};
use bms_replay::replay_data::ReplayData;
use bms_rule::ScoreData;

/// Data shared across game states during a play session.
#[derive(Debug, Clone)]
pub struct PlayerResource {
    /// Currently loaded BMS chart (None if nothing is loaded).
    pub bms_model: Option<BmsModel>,
    /// Score data for the current play session.
    pub score_data: ScoreData,
    /// Active play mode.
    pub play_mode: PlayMode,
    /// Player configuration snapshot.
    #[allow(dead_code)]
    pub player_config: PlayerConfig,
    /// Original gauge option (saved at Decide, may be modified during Play).
    pub org_gauge_option: i32,
    /// BMS file's parent directory (for resolving WAV paths).
    pub bms_dir: Option<PathBuf>,

    // --- Play result fields (populated by PlayState shutdown) ---
    /// Gauge log: per-gauge-type values recorded every 500ms during play.
    pub gauge_log: Vec<Vec<f32>>,
    /// Maximum combo achieved.
    pub maxcombo: i32,
    /// Whether this score should be saved (false for autoplay/replay).
    pub update_score: bool,
    /// Assist option flags.
    #[allow(dead_code)]
    pub assist: i32,

    // --- Result state fields ---
    /// Previous best score from DB (loaded in ResultState::create).
    pub oldscore: ScoreData,
    /// Accumulated course scores per stage (None when not in course mode).
    pub course_score_data: Option<Vec<ScoreData>>,
    /// Accumulated course replays.
    #[allow(dead_code)] // Reserved for course mode replay system
    pub course_replays: Vec<ReplayData>,
    /// Accumulated course gauge logs.
    #[allow(dead_code)] // Reserved for course mode gauge display
    pub course_gauges: Vec<Vec<f32>>,
    /// Current play's replay data.
    #[allow(dead_code)] // Reserved for replay saving system
    pub replay_data: Option<ReplayData>,
    /// Flag set by KeyConfig/SkinConfig shutdown to request config file save.
    pub config_save_requested: bool,
}

impl Default for PlayerResource {
    fn default() -> Self {
        Self {
            bms_model: None,
            score_data: ScoreData::default(),
            play_mode: PlayMode::Beat7K,
            player_config: PlayerConfig::default(),
            org_gauge_option: 0,
            bms_dir: None,
            gauge_log: Vec::new(),
            maxcombo: 0,
            update_score: false,
            assist: 0,
            oldscore: ScoreData::default(),
            course_score_data: None,
            course_replays: Vec::new(),
            course_gauges: Vec::new(),
            replay_data: None,
            config_save_requested: false,
        }
    }
}

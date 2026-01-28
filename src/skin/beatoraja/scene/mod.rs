//! Scene-specific skin handling
//!
//! Each scene type (play, select, result) has specific elements and behaviors.

pub mod decide;
pub mod play;
pub mod result;
pub mod select;

pub use decide::{DecideSkinConfig, DecideSongInfo, LoadingProgressConfig};
pub use play::{LaneConfig, LaneNoteSkin, NoteType, PlaySkinConfig, get_note_image_id};
pub use result::{
    ClearResultConfig, JudgeStatsConfig, RankingConfig, ResultSkinConfig, ResultSongInfo,
    ScoreDisplayConfig,
};
pub use select::{
    DifficultyConfig, FolderConfig, SelectSkinConfig, SongInfoConfig, SongListConfig,
};

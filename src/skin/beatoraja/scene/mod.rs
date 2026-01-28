//! Scene-specific skin handling
//!
//! Each scene type (play, select, result) has specific elements and behaviors.

pub mod play;
pub mod select;

pub use play::{LaneConfig, LaneNoteSkin, NoteType, PlaySkinConfig, get_note_image_id};
pub use select::{
    DifficultyConfig, FolderConfig, SelectSkinConfig, SongInfoConfig, SongListConfig,
};

// Future modules:
// pub mod decide;
// pub mod result;

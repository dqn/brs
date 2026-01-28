//! Scene-specific skin handling
//!
//! Each scene type (play, select, result) has specific elements and behaviors.

pub mod play;

pub use play::{LaneConfig, LaneNoteSkin, NoteType, PlaySkinConfig, get_note_image_id};

// Future modules:
// pub mod select;
// pub mod decide;
// pub mod result;

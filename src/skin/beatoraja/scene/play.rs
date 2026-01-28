//! Play scene skin handling
//!
//! Manages note rendering, gauge display, and other play-specific elements.

use super::super::types::{BeatorajaSkin, NoteSkin};

/// Lane configuration for play skins
#[derive(Debug, Clone)]
pub struct LaneConfig {
    /// Lane index
    pub lane: usize,
    /// X position of lane
    pub x: f32,
    /// Y position of lane top
    pub y: f32,
    /// Lane width
    pub width: f32,
    /// Lane height (from top to judge line)
    pub height: f32,
}

/// Note skin configuration for a single lane
#[derive(Debug, Clone, Default)]
pub struct LaneNoteSkin {
    /// Normal note image ID
    pub note_id: Option<i32>,
    /// LN start image ID
    pub ln_start_id: Option<i32>,
    /// LN body image ID
    pub ln_body_id: Option<i32>,
    /// LN end image ID
    pub ln_end_id: Option<i32>,
    /// LN active body image ID
    pub ln_active_id: Option<i32>,
    /// Mine note image ID
    pub mine_id: Option<i32>,
    /// Hidden note image ID (HIDDEN+)
    pub hidden_id: Option<i32>,
    /// Processed note image ID
    pub processed_id: Option<i32>,
}

/// Play skin configuration extracted from beatoraja skin
#[derive(Debug, Clone)]
pub struct PlaySkinConfig {
    /// Skin width
    pub width: i32,
    /// Skin height
    pub height: i32,
    /// Number of lanes
    pub lane_count: usize,
    /// Lane configurations
    pub lanes: Vec<LaneConfig>,
    /// Note skins per lane
    pub note_skins: Vec<LaneNoteSkin>,
    /// Judge line Y position
    pub judge_line_y: f32,
    /// Whether to use LN body tiling (vs stretching)
    pub ln_body_tile: bool,
}

impl PlaySkinConfig {
    /// Create from a beatoraja skin
    pub fn from_skin(skin: &BeatorajaSkin) -> Option<Self> {
        let note_skin = skin.note.as_ref()?;
        let lane_count = count_lanes(note_skin);

        if lane_count == 0 {
            return None;
        }

        // Extract lane configurations from note destinations
        let lanes = extract_lane_configs(note_skin, lane_count);
        let note_skins = extract_note_skins(note_skin, lane_count);
        let judge_line_y = extract_judge_line_y(note_skin);

        Some(Self {
            width: skin.header.w,
            height: skin.header.h,
            lane_count,
            lanes,
            note_skins,
            judge_line_y,
            ln_body_tile: false, // Default to stretching
        })
    }

    /// Get note skin for a lane
    pub fn get_lane_skin(&self, lane: usize) -> Option<&LaneNoteSkin> {
        self.note_skins.get(lane)
    }
}

/// Count the number of lanes from note definitions
fn count_lanes(note_skin: &NoteSkin) -> usize {
    note_skin
        .note
        .iter()
        .map(|n| n.lane as usize)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0)
}

/// Extract lane configurations from note destinations
fn extract_lane_configs(note_skin: &NoteSkin, lane_count: usize) -> Vec<LaneConfig> {
    let mut configs = Vec::with_capacity(lane_count);

    for lane in 0..lane_count {
        // Find the note element for this lane
        if let Some(note) = note_skin.note.iter().find(|n| n.lane as usize == lane) {
            if let Some(dst) = note.dst.first() {
                configs.push(LaneConfig {
                    lane,
                    x: dst.x as f32,
                    y: dst.y as f32,
                    width: dst.w as f32,
                    height: dst.h as f32,
                });
            } else {
                configs.push(default_lane_config(lane));
            }
        } else {
            configs.push(default_lane_config(lane));
        }
    }

    configs
}

fn default_lane_config(lane: usize) -> LaneConfig {
    LaneConfig {
        lane,
        x: 0.0,
        y: 0.0,
        width: 50.0,
        height: 500.0,
    }
}

/// Extract note skins for each lane
fn extract_note_skins(note_skin: &NoteSkin, lane_count: usize) -> Vec<LaneNoteSkin> {
    let mut skins = vec![LaneNoteSkin::default(); lane_count];

    for note in &note_skin.note {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.note_id = Some(note.id);
        }
    }

    for note in &note_skin.lnstart {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.ln_start_id = Some(note.id);
        }
    }

    for note in &note_skin.lnbody {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.ln_body_id = Some(note.id);
        }
    }

    for note in &note_skin.lnend {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.ln_end_id = Some(note.id);
        }
    }

    for note in &note_skin.lnactive {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.ln_active_id = Some(note.id);
        }
    }

    for note in &note_skin.mine {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.mine_id = Some(note.id);
        }
    }

    for note in &note_skin.hidden {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.hidden_id = Some(note.id);
        }
    }

    for note in &note_skin.processed {
        if let Some(skin) = skins.get_mut(note.lane as usize) {
            skin.processed_id = Some(note.id);
        }
    }

    skins
}

/// Extract judge line Y position from note destinations
fn extract_judge_line_y(note_skin: &NoteSkin) -> f32 {
    // The judge line is typically at the bottom of the note destination
    note_skin
        .note
        .first()
        .and_then(|n| n.dst.first())
        .map(|d| (d.y + d.h) as f32)
        .unwrap_or(500.0)
}

/// Note type for skin rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Normal,
    LnStart,
    LnBody,
    LnEnd,
    LnActive,
    Mine,
    Hidden,
    Processed,
}

impl NoteType {
    /// Convert from BMS NoteType to skin NoteType
    pub fn from_bms_note_type(bms_type: crate::bms::NoteType) -> Self {
        match bms_type {
            crate::bms::NoteType::Normal => Self::Normal,
            crate::bms::NoteType::LongStart => Self::LnStart,
            crate::bms::NoteType::LongEnd => Self::LnEnd,
            crate::bms::NoteType::Invisible => Self::Hidden,
            crate::bms::NoteType::Landmine => Self::Mine,
        }
    }
}

/// Get image ID for a note type and lane
pub fn get_note_image_id(config: &PlaySkinConfig, lane: usize, note_type: NoteType) -> Option<i32> {
    let skin = config.get_lane_skin(lane)?;

    match note_type {
        NoteType::Normal => skin.note_id,
        NoteType::LnStart => skin.ln_start_id.or(skin.note_id),
        NoteType::LnBody => skin.ln_body_id,
        NoteType::LnEnd => skin.ln_end_id.or(skin.note_id),
        NoteType::LnActive => skin.ln_active_id.or(skin.ln_body_id),
        NoteType::Mine => skin.mine_id,
        NoteType::Hidden => skin.hidden_id.or(skin.note_id),
        NoteType::Processed => skin.processed_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::beatoraja::types::{Destination, NoteElement, NoteSkin};

    fn create_test_note(lane: i32, id: i32, x: i32, w: i32) -> NoteElement {
        NoteElement {
            lane,
            id,
            dst: vec![Destination {
                time: 0,
                x,
                y: 0,
                w,
                h: 500,
                ..Default::default()
            }],
        }
    }

    #[test]
    fn test_count_lanes() {
        let note_skin = NoteSkin {
            note: vec![
                create_test_note(0, 1, 0, 50),
                create_test_note(1, 2, 50, 50),
                create_test_note(7, 8, 350, 50),
            ],
            ..Default::default()
        };

        assert_eq!(count_lanes(&note_skin), 8);
    }

    #[test]
    fn test_extract_lane_configs() {
        let note_skin = NoteSkin {
            note: vec![
                create_test_note(0, 1, 0, 60),
                create_test_note(1, 2, 60, 40),
            ],
            ..Default::default()
        };

        let configs = extract_lane_configs(&note_skin, 2);
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].x, 0.0);
        assert_eq!(configs[0].width, 60.0);
        assert_eq!(configs[1].x, 60.0);
        assert_eq!(configs[1].width, 40.0);
    }

    #[test]
    fn test_extract_note_skins() {
        let note_skin = NoteSkin {
            note: vec![create_test_note(0, 10, 0, 50)],
            lnstart: vec![create_test_note(0, 20, 0, 50)],
            lnbody: vec![create_test_note(0, 30, 0, 50)],
            lnend: vec![create_test_note(0, 40, 0, 50)],
            ..Default::default()
        };

        let skins = extract_note_skins(&note_skin, 1);
        assert_eq!(skins.len(), 1);
        assert_eq!(skins[0].note_id, Some(10));
        assert_eq!(skins[0].ln_start_id, Some(20));
        assert_eq!(skins[0].ln_body_id, Some(30));
        assert_eq!(skins[0].ln_end_id, Some(40));
    }

    #[test]
    fn test_get_note_image_id_fallback() {
        let config = PlaySkinConfig {
            width: 1920,
            height: 1080,
            lane_count: 1,
            lanes: vec![default_lane_config(0)],
            note_skins: vec![LaneNoteSkin {
                note_id: Some(10),
                ln_start_id: None, // Will fall back to note_id
                ..Default::default()
            }],
            judge_line_y: 500.0,
            ln_body_tile: false,
        };

        // LnStart without specific ID falls back to note_id
        assert_eq!(get_note_image_id(&config, 0, NoteType::LnStart), Some(10));

        // Normal note
        assert_eq!(get_note_image_id(&config, 0, NoteType::Normal), Some(10));
    }
}

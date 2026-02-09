use serde::{Deserialize, Serialize};

use bms_model::{BmsModel, NoteType, PlayMode};

// Feature flags (matches Java SongData.FEATURE_*)
pub const FEATURE_UNDEFINEDLN: i32 = 1;
pub const FEATURE_MINENOTE: i32 = 2;
pub const FEATURE_RANDOM: i32 = 4;
pub const FEATURE_LONGNOTE: i32 = 8;
pub const FEATURE_CHARGENOTE: i32 = 16;
pub const FEATURE_HELLCHARGENOTE: i32 = 32;
pub const FEATURE_STOPSEQUENCE: i32 = 64;
pub const FEATURE_SCROLL: i32 = 128;

// Content flags (matches Java SongData.CONTENT_*)
pub const CONTENT_TEXT: i32 = 1;
pub const CONTENT_BGA: i32 = 2;
pub const CONTENT_PREVIEW: i32 = 4;
pub const CONTENT_NOKEYSOUND: i32 = 128;

// Favorite flags
pub const FAVORITE_SONG: i32 = 1;
pub const FAVORITE_CHART: i32 = 2;
pub const INVISIBLE_SONG: i32 = 4;
pub const INVISIBLE_CHART: i32 = 8;

/// Song data stored in the song database.
///
/// Corresponds to Java `SongData` with 29 DB columns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SongData {
    pub md5: String,
    pub sha256: String,
    pub title: String,
    pub subtitle: String,
    pub genre: String,
    pub artist: String,
    pub subartist: String,
    pub tag: String,
    pub path: String,
    pub folder: String,
    pub stagefile: String,
    pub banner: String,
    pub backbmp: String,
    pub preview: String,
    pub parent: String,
    pub level: i32,
    pub difficulty: i32,
    pub maxbpm: i32,
    pub minbpm: i32,
    pub length: i32,
    pub mode: i32,
    pub judge: i32,
    pub feature: i32,
    pub content: i32,
    pub date: i32,
    pub favorite: i32,
    pub adddate: i32,
    pub notes: i32,
    pub charthash: String,
}

impl SongData {
    /// Create SongData from a parsed BmsModel.
    pub fn from_model(model: &BmsModel, contains_txt: bool) -> Self {
        let mut content = if contains_txt { CONTENT_TEXT } else { 0 };
        let mut feature = 0i32;

        for note in &model.notes {
            match note.note_type {
                NoteType::LongNote => feature |= FEATURE_LONGNOTE,
                NoteType::ChargeNote => feature |= FEATURE_CHARGENOTE,
                NoteType::HellChargeNote => feature |= FEATURE_HELLCHARGENOTE,
                NoteType::Mine => feature |= FEATURE_MINENOTE,
                _ => {}
            }
        }

        if !model.stop_events.is_empty() {
            feature |= FEATURE_STOPSEQUENCE;
        }

        // CONTENT_BGA: check if bmp_defs is non-empty
        if !model.bmp_defs.is_empty() {
            content |= CONTENT_BGA;
        }

        // CONTENT_NOKEYSOUND: length >= 30000ms and few wav defs
        let length_ms = (model.total_time_us / 1000) as i32;
        if length_ms >= 30000 && model.wav_defs.len() as i32 <= (length_ms / 50000) + 3 {
            content |= CONTENT_NOKEYSOUND;
        }

        Self {
            md5: model.md5.clone(),
            sha256: model.sha256.clone(),
            title: model.title.clone(),
            subtitle: model.subtitle.clone(),
            genre: model.genre.clone(),
            artist: model.artist.clone(),
            subartist: model.sub_artist.clone(),
            tag: String::new(),
            path: String::new(),
            folder: String::new(),
            stagefile: model.stage_file.clone(),
            banner: model.banner.clone(),
            backbmp: model.back_bmp.clone(),
            preview: model.preview.clone(),
            parent: String::new(),
            level: model.play_level,
            difficulty: model.difficulty,
            maxbpm: model.max_bpm() as i32,
            minbpm: model.min_bpm() as i32,
            length: length_ms,
            mode: model.mode.mode_id(),
            judge: model.judge_rank,
            feature,
            content,
            date: 0,
            favorite: 0,
            adddate: 0,
            notes: model.total_notes() as i32,
            charthash: String::new(),
        }
    }

    /// Validate that required fields are present.
    pub fn validate(&self) -> bool {
        if self.title.is_empty() {
            return false;
        }
        if self.md5.is_empty() && self.sha256.is_empty() {
            return false;
        }
        true
    }

    pub fn full_title(&self) -> String {
        if self.subtitle.is_empty() {
            self.title.clone()
        } else {
            format!("{} {}", self.title, self.subtitle)
        }
    }

    pub fn has_random_sequence(&self) -> bool {
        self.feature & FEATURE_RANDOM != 0
    }

    pub fn has_mine_note(&self) -> bool {
        self.feature & FEATURE_MINENOTE != 0
    }

    pub fn has_undefined_long_note(&self) -> bool {
        self.feature & FEATURE_UNDEFINEDLN != 0
    }

    pub fn has_long_note(&self) -> bool {
        self.feature & FEATURE_LONGNOTE != 0
    }

    pub fn has_charge_note(&self) -> bool {
        self.feature & FEATURE_CHARGENOTE != 0
    }

    pub fn has_hell_charge_note(&self) -> bool {
        self.feature & FEATURE_HELLCHARGENOTE != 0
    }

    pub fn has_any_long_note(&self) -> bool {
        self.feature
            & (FEATURE_UNDEFINEDLN | FEATURE_LONGNOTE | FEATURE_CHARGENOTE | FEATURE_HELLCHARGENOTE)
            != 0
    }

    pub fn has_stop_sequence(&self) -> bool {
        self.feature & FEATURE_STOPSEQUENCE != 0
    }

    pub fn has_document(&self) -> bool {
        self.content & CONTENT_TEXT != 0
    }

    pub fn has_bga(&self) -> bool {
        self.content & CONTENT_BGA != 0
    }

    pub fn play_mode(&self) -> Option<PlayMode> {
        PlayMode::from_mode_id(self.mode)
    }
}

/// Read a SongData from a rusqlite row.
impl SongData {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            md5: row.get("md5")?,
            sha256: row.get("sha256")?,
            title: row.get("title")?,
            subtitle: row
                .get::<_, Option<String>>("subtitle")?
                .unwrap_or_default(),
            genre: row.get::<_, Option<String>>("genre")?.unwrap_or_default(),
            artist: row.get::<_, Option<String>>("artist")?.unwrap_or_default(),
            subartist: row
                .get::<_, Option<String>>("subartist")?
                .unwrap_or_default(),
            tag: row.get::<_, Option<String>>("tag")?.unwrap_or_default(),
            path: row.get("path")?,
            folder: row.get::<_, Option<String>>("folder")?.unwrap_or_default(),
            stagefile: row
                .get::<_, Option<String>>("stagefile")?
                .unwrap_or_default(),
            banner: row.get::<_, Option<String>>("banner")?.unwrap_or_default(),
            backbmp: row.get::<_, Option<String>>("backbmp")?.unwrap_or_default(),
            preview: row.get::<_, Option<String>>("preview")?.unwrap_or_default(),
            parent: row.get::<_, Option<String>>("parent")?.unwrap_or_default(),
            level: row.get::<_, Option<i32>>("level")?.unwrap_or(0),
            difficulty: row.get::<_, Option<i32>>("difficulty")?.unwrap_or(0),
            maxbpm: row.get::<_, Option<i32>>("maxbpm")?.unwrap_or(0),
            minbpm: row.get::<_, Option<i32>>("minbpm")?.unwrap_or(0),
            length: row.get::<_, Option<i32>>("length")?.unwrap_or(0),
            mode: row.get::<_, Option<i32>>("mode")?.unwrap_or(0),
            judge: row.get::<_, Option<i32>>("judge")?.unwrap_or(0),
            feature: row.get::<_, Option<i32>>("feature")?.unwrap_or(0),
            content: row.get::<_, Option<i32>>("content")?.unwrap_or(0),
            date: row.get::<_, Option<i32>>("date")?.unwrap_or(0),
            favorite: row.get::<_, Option<i32>>("favorite")?.unwrap_or(0),
            adddate: row.get::<_, Option<i32>>("adddate")?.unwrap_or(0),
            notes: row.get::<_, Option<i32>>("notes")?.unwrap_or(0),
            charthash: row
                .get::<_, Option<String>>("charthash")?
                .unwrap_or_default(),
        })
    }
}

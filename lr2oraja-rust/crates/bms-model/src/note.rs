use serde::{Deserialize, Serialize};

/// LN mode defined by #LNTYPE header
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LnType {
    #[default]
    LongNote = 1,
    ChargeNote = 2,
    HellChargeNote = 3,
}

/// The type of a note
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoteType {
    Normal,
    LongNote,
    ChargeNote,
    HellChargeNote,
    Mine,
    Invisible,
}

/// A single note in the chart
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Lane index (0-indexed)
    pub lane: usize,
    /// Note type
    pub note_type: NoteType,
    /// Start time in microseconds
    pub time_us: i64,
    /// End time in microseconds (for LN/CN/HCN only, 0 for others)
    pub end_time_us: i64,
    /// WAV definition ID
    pub wav_id: u16,
    /// End WAV definition ID (for LN end)
    pub end_wav_id: u16,
    /// Damage value (for mine notes)
    pub damage: i32,
    /// Index of paired end note in the notes vec (for LN start only, usize::MAX if none)
    pub pair_index: usize,
    /// Audio slice start time in microseconds (bmson sound slicing, 0 = no slice)
    pub micro_starttime: i64,
    /// Audio slice duration in microseconds (bmson sound slicing, 0 = full length)
    pub micro_duration: i64,
}

impl Note {
    /// Create a minimal note for key sound playback only.
    pub fn keysound(wav_id: u16) -> Self {
        Self {
            lane: 0,
            note_type: NoteType::Normal,
            time_us: 0,
            end_time_us: 0,
            wav_id,
            end_wav_id: 0,
            damage: 0,
            pair_index: usize::MAX,
            micro_starttime: 0,
            micro_duration: 0,
        }
    }

    pub fn normal(lane: usize, time_us: i64, wav_id: u16) -> Self {
        Self {
            lane,
            note_type: NoteType::Normal,
            time_us,
            end_time_us: 0,
            wav_id,
            end_wav_id: 0,
            damage: 0,
            pair_index: usize::MAX,
            micro_starttime: 0,
            micro_duration: 0,
        }
    }

    pub fn long_note(
        lane: usize,
        time_us: i64,
        end_time_us: i64,
        wav_id: u16,
        end_wav_id: u16,
        ln_type: LnType,
    ) -> Self {
        let note_type = match ln_type {
            LnType::LongNote => NoteType::LongNote,
            LnType::ChargeNote => NoteType::ChargeNote,
            LnType::HellChargeNote => NoteType::HellChargeNote,
        };
        Self {
            lane,
            note_type,
            time_us,
            end_time_us,
            wav_id,
            end_wav_id,
            damage: 0,
            pair_index: usize::MAX,
            micro_starttime: 0,
            micro_duration: 0,
        }
    }

    pub fn mine(lane: usize, time_us: i64, wav_id: u16, damage: i32) -> Self {
        Self {
            lane,
            note_type: NoteType::Mine,
            time_us,
            end_time_us: 0,
            wav_id,
            end_wav_id: 0,
            damage,
            pair_index: usize::MAX,
            micro_starttime: 0,
            micro_duration: 0,
        }
    }

    pub fn invisible(lane: usize, time_us: i64, wav_id: u16) -> Self {
        Self {
            lane,
            note_type: NoteType::Invisible,
            time_us,
            end_time_us: 0,
            wav_id,
            end_wav_id: 0,
            damage: 0,
            pair_index: usize::MAX,
            micro_starttime: 0,
            micro_duration: 0,
        }
    }

    pub fn is_long_note(&self) -> bool {
        matches!(
            self.note_type,
            NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote
        )
    }

    pub fn is_playable(&self) -> bool {
        !matches!(self.note_type, NoteType::Mine | NoteType::Invisible)
    }

    /// Duration in microseconds (for LN types)
    pub fn duration_us(&self) -> i64 {
        self.end_time_us - self.time_us
    }
}

/// A background note (BGM channel 0x01 or bmson BGM).
///
/// Separate from `Note` because BG notes don't have lanes or note types —
/// they only carry a wav_id and timing info for audio playback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BgNote {
    /// WAV definition ID
    pub wav_id: u16,
    /// Trigger time in microseconds
    pub time_us: i64,
    /// Audio slice start time in microseconds (bmson, 0 = no slice)
    pub micro_starttime: i64,
    /// Audio slice duration in microseconds (bmson, 0 = full length)
    pub micro_duration: i64,
}

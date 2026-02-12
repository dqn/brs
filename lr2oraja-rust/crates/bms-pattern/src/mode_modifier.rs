// Mode modifier — changes play mode (e.g., 7K -> 9K).
//
// Ported from Java: ModeModifier.java
//
// Currently only the SEVEN_TO_NINE algorithm (Beat7K -> PopN9K) is implemented,
// matching the Java source which also only has this algorithm.

use std::collections::BTreeMap;

use bms_model::{BmsModel, NoteType, PlayMode};

use crate::modifier::{AssistLevel, PatternModifier};

/// Configuration for 7-to-9 scratch lane placement.
///
/// Maps to Java `config.getSevenToNinePattern()` (0-6).
///
/// Pattern determines which 9K button positions the scratch lane
/// and the "rest" (extra) lane map to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SevenToNinePattern {
    /// Pattern 0: default (sc=lane1, key=lane2, rest=lane0)
    Default,
    /// Pattern 1: SC→0, KEY→1..7, REST→8
    Sc1Key2To8,
    /// Pattern 2: SC→0, KEY→2..8, REST→1
    Sc1Key3To9,
    /// Pattern 3: SC→1, KEY→2..8, REST→0 (also default fallback)
    Sc2Key3To9,
    /// Pattern 4: SC→7, KEY→0..6, REST→8
    Sc8Key1To7,
    /// Pattern 5: SC→8, KEY→0..6, REST→7
    Sc9Key1To7,
    /// Pattern 6: SC→8, KEY→1..7, REST→0
    Sc9Key2To8,
}

impl SevenToNinePattern {
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Sc1Key2To8,
            2 => Self::Sc1Key3To9,
            4 => Self::Sc8Key1To7,
            5 => Self::Sc9Key1To7,
            6 => Self::Sc9Key2To8,
            _ => Self::Sc2Key3To9, // 0 and 3 both map to default
        }
    }

    /// Returns (sc_lane, key_lane_start, rest_lane) for the 9K layout.
    fn layout(self) -> (usize, usize, usize) {
        match self {
            Self::Default | Self::Sc2Key3To9 => (1, 2, 0),
            Self::Sc1Key2To8 => (0, 1, 8),
            Self::Sc1Key3To9 => (0, 2, 1),
            Self::Sc8Key1To7 => (7, 0, 8),
            Self::Sc9Key1To7 => (8, 0, 7),
            Self::Sc9Key2To8 => (8, 1, 0),
        }
    }
}

/// Configuration for 7-to-9 scratch note distribution.
///
/// Maps to Java `config.getSevenToNineType()` (0-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SevenToNineType {
    /// Type 0: scratch always goes to sc_lane, rest gets nothing
    Passthrough,
    /// Type 1: alternate to avoid repeated presses (with threshold)
    AlternateAvoid,
    /// Type 2: alternate (always choose the lane with older last note)
    Alternate,
}

impl SevenToNineType {
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::AlternateAvoid,
            2 => Self::Alternate,
            _ => Self::Passthrough,
        }
    }
}

/// Changes the play mode of a chart (e.g., Beat7K -> PopN9K).
///
/// Java: `ModeModifier`
pub struct ModeModifier {
    pub before_mode: PlayMode,
    pub after_mode: PlayMode,
    pub seven_to_nine_pattern: SevenToNinePattern,
    pub seven_to_nine_type: SevenToNineType,
    /// H-RANDOM threshold BPM (0 = disabled). Used to compute duration threshold.
    pub hran_threshold_bpm: f64,
}

impl ModeModifier {
    pub fn new(before_mode: PlayMode, after_mode: PlayMode) -> Self {
        Self {
            before_mode,
            after_mode,
            seven_to_nine_pattern: SevenToNinePattern::Default,
            seven_to_nine_type: SevenToNineType::Passthrough,
            hran_threshold_bpm: 0.0,
        }
    }

    pub fn with_pattern(mut self, pattern: SevenToNinePattern) -> Self {
        self.seven_to_nine_pattern = pattern;
        self
    }

    pub fn with_type(mut self, seven_type: SevenToNineType) -> Self {
        self.seven_to_nine_type = seven_type;
        self
    }

    pub fn with_hran_threshold_bpm(mut self, bpm: f64) -> Self {
        self.hran_threshold_bpm = bpm;
        self
    }
}

impl PatternModifier for ModeModifier {
    fn modify(&mut self, model: &mut BmsModel) {
        if self.before_mode == PlayMode::Beat7K && self.after_mode == PlayMode::PopN9K {
            self.seven_to_nine(model);
        }
        // Other mode conversions would go here when implemented
    }

    fn assist_level(&self) -> AssistLevel {
        AssistLevel::LightAssist
    }
}

impl ModeModifier {
    /// Convert Beat7K chart to PopN9K.
    ///
    /// The 7 key lanes (0-6) are mapped to 7 of the 9 PopN buttons.
    /// The scratch lane (7) is distributed between the scratch lane and
    /// rest lane based on the seven_to_nine_type setting.
    fn seven_to_nine(&self, model: &mut BmsModel) {
        let (sc_lane, key_lane, rest_lane) = self.seven_to_nine_pattern.layout();

        // Compute duration threshold from BPM
        let hran_threshold: i32 = if self.hran_threshold_bpm <= 0.0 {
            0
        } else {
            (15000.0_f64 / self.hran_threshold_bpm).ceil() as i32
        };

        let after_key_count = self.after_mode.key_count(); // 9

        // State tracking
        let mut ln_active = vec![-1i32; after_key_count];
        let mut last_note_time = vec![-100i32; after_key_count];
        let mut end_ln_note_time = vec![-1i32; after_key_count];

        // Group notes by time_us
        let mut time_groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (idx, note) in model.notes.iter().enumerate() {
            time_groups.entry(note.time_us).or_default().push(idx);
        }

        // Process each timeline
        for (&_time_us, indices) in &time_groups {
            let time_ms = (_time_us / 1000) as i32;

            // Build current mapping for this timeline
            let mut mapping = vec![usize::MAX; after_key_count];

            // Map 7 key lanes to their 9K positions
            for i in 0..7 {
                mapping[i + key_lane] = i;
            }

            // Determine sc/rest lane mapping
            // Java source lane 7 = scratch, source lane 8 = "empty" (beyond 7K)
            let (sc_source, rest_source) = self.determine_sc_rest_mapping(
                sc_lane,
                rest_lane,
                &ln_active,
                &last_note_time,
                time_ms,
                hran_threshold,
            );
            mapping[sc_lane] = sc_source;
            mapping[rest_lane] = rest_source;

            // Apply mapping: remap notes at this timeline
            for &idx in indices {
                let note = &model.notes[idx];
                let old_lane = note.lane;

                // Find which new lane maps from this old lane
                let new_lane = mapping
                    .iter()
                    .position(|&src| src == old_lane)
                    .unwrap_or(old_lane);

                let note = &model.notes[idx];
                let is_ln = note.is_long_note();
                let is_ln_end = is_ln && note.end_time_us == 0;

                // Update state
                if is_ln {
                    if is_ln_end && time_ms == end_ln_note_time[new_lane] {
                        ln_active[new_lane] = -1;
                        end_ln_note_time[new_lane] = -1;
                    } else {
                        ln_active[new_lane] = old_lane as i32;
                        if !is_ln_end {
                            end_ln_note_time[new_lane] =
                                (model.notes[idx].end_time_us / 1000) as i32;
                        }
                        last_note_time[new_lane] = time_ms;
                    }
                } else if note.note_type != NoteType::Invisible {
                    last_note_time[new_lane] = time_ms;
                }

                model.notes[idx].lane = new_lane;
            }
        }

        // Update model mode
        model.mode = self.after_mode;
    }

    /// Determine which source lanes sc_lane and rest_lane should read from.
    ///
    /// Returns (sc_source, rest_source) where source 7 = scratch, 8 = empty.
    fn determine_sc_rest_mapping(
        &self,
        sc_lane: usize,
        rest_lane: usize,
        ln_active: &[i32],
        last_note_time: &[i32],
        now: i32,
        duration: i32,
    ) -> (usize, usize) {
        // If LN is active on either lane, maintain the current assignment
        if ln_active[sc_lane] != -1 || ln_active[rest_lane] != -1 {
            if ln_active[sc_lane] == 7 {
                (7, 8)
            } else {
                (8, 7)
            }
        } else {
            match self.seven_to_nine_type {
                SevenToNineType::Passthrough => (7, 8),
                SevenToNineType::AlternateAvoid => {
                    // Prefer the lane with older or equal last note time,
                    // but only if duration threshold is met
                    if now - last_note_time[sc_lane] > duration
                        || now - last_note_time[sc_lane] >= now - last_note_time[rest_lane]
                    {
                        (7, 8)
                    } else {
                        (8, 7)
                    }
                }
                SevenToNineType::Alternate => {
                    // Always prefer the lane with older last note
                    if now - last_note_time[sc_lane] >= now - last_note_time[rest_lane] {
                        (7, 8)
                    } else {
                        (8, 7)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::{LnType, Note};

    fn make_model(notes: Vec<Note>) -> BmsModel {
        BmsModel {
            mode: PlayMode::Beat7K,
            notes,
            ..Default::default()
        }
    }

    #[test]
    fn test_seven_to_nine_basic_key_mapping() {
        // Default pattern (sc=1, key=2, rest=0): keys 0-6 -> lanes 2-8
        let notes = vec![
            Note::normal(0, 1000, 1), // key 0 -> lane 2
            Note::normal(3, 1000, 2), // key 3 -> lane 5
            Note::normal(6, 1000, 3), // key 6 -> lane 8
        ];
        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K);
        modifier.modify(&mut model);

        assert_eq!(model.mode, PlayMode::PopN9K);
        assert_eq!(model.notes[0].lane, 2); // key 0 -> lane 2
        assert_eq!(model.notes[1].lane, 5); // key 3 -> lane 5
        assert_eq!(model.notes[2].lane, 8); // key 6 -> lane 8
    }

    #[test]
    fn test_seven_to_nine_scratch_passthrough() {
        // Scratch (lane 7) -> sc_lane (1) in default pattern
        let notes = vec![Note::normal(7, 1000, 1)]; // scratch
        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K);
        modifier.modify(&mut model);

        assert_eq!(model.notes[0].lane, 1); // scratch -> lane 1
    }

    #[test]
    fn test_seven_to_nine_pattern_sc1_key2_to8() {
        // Pattern 1: SC→0, KEY→1..7, REST→8
        let notes = vec![
            Note::normal(0, 1000, 1), // key 0 -> lane 1
            Note::normal(7, 1000, 2), // scratch -> lane 0
        ];
        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K)
            .with_pattern(SevenToNinePattern::Sc1Key2To8);
        modifier.modify(&mut model);

        assert_eq!(model.notes[0].lane, 1); // key 0 -> lane 1
        assert_eq!(model.notes[1].lane, 0); // scratch -> lane 0
    }

    #[test]
    fn test_seven_to_nine_pattern_sc9_key1_to7() {
        // Pattern 5: SC→8, KEY→0..6, REST→7
        let notes = vec![
            Note::normal(0, 1000, 1), // key 0 -> lane 0
            Note::normal(6, 1000, 2), // key 6 -> lane 6
            Note::normal(7, 1000, 3), // scratch -> lane 8
        ];
        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K)
            .with_pattern(SevenToNinePattern::Sc9Key1To7);
        modifier.modify(&mut model);

        assert_eq!(model.notes[0].lane, 0); // key 0 -> lane 0
        assert_eq!(model.notes[1].lane, 6); // key 6 -> lane 6
        assert_eq!(model.notes[2].lane, 8); // scratch -> lane 8
    }

    #[test]
    fn test_seven_to_nine_alternate_type() {
        // With alternate type, scratch alternates between sc_lane and rest_lane
        let notes = vec![
            Note::normal(7, 1000_000, 1), // scratch at 1000ms
            Note::normal(7, 2000_000, 2), // scratch at 2000ms
        ];
        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K)
            .with_type(SevenToNineType::Alternate);
        modifier.modify(&mut model);

        // First scratch goes to sc_lane (1), second should alternate to rest_lane (0)
        assert_eq!(model.notes[0].lane, 1);
        assert_eq!(model.notes[1].lane, 0);
    }

    #[test]
    fn test_seven_to_nine_assist_level() {
        let modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K);
        assert_eq!(modifier.assist_level(), AssistLevel::LightAssist);
    }

    #[test]
    fn test_seven_to_nine_with_ln() {
        let mut notes = vec![
            Note::long_note(7, 1000_000, 2000_000, 1, 2, LnType::LongNote), // LN start
            Note::long_note(7, 2000_000, 0, 2, 0, LnType::LongNote),        // LN end
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;

        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K);
        modifier.modify(&mut model);

        // Both should be on sc_lane (1)
        assert_eq!(model.notes[0].lane, 1);
        assert_eq!(model.notes[1].lane, 1);
    }

    #[test]
    fn test_seven_to_nine_mixed_notes() {
        let notes = vec![
            Note::normal(0, 1000_000, 1),
            Note::normal(3, 1000_000, 2),
            Note::normal(7, 1000_000, 3), // scratch
            Note::mine(2, 2000_000, 4, 10),
        ];
        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat7K, PlayMode::PopN9K);
        modifier.modify(&mut model);

        assert_eq!(model.mode, PlayMode::PopN9K);
        assert_eq!(model.notes.len(), 4);
    }

    #[test]
    fn test_non_seven_to_nine_noop() {
        let notes = vec![Note::normal(0, 1000, 1)];
        let mut model = make_model(notes);
        let mut modifier = ModeModifier::new(PlayMode::Beat5K, PlayMode::Beat7K);
        modifier.modify(&mut model);

        // No algorithm for this conversion, should not modify
        assert_eq!(model.mode, PlayMode::Beat7K); // mode unchanged (before is Beat5K)
        assert_eq!(model.notes[0].lane, 0);
    }

    #[test]
    fn test_pattern_from_id() {
        assert_eq!(
            SevenToNinePattern::from_id(0),
            SevenToNinePattern::Sc2Key3To9
        );
        assert_eq!(
            SevenToNinePattern::from_id(1),
            SevenToNinePattern::Sc1Key2To8
        );
        assert_eq!(
            SevenToNinePattern::from_id(2),
            SevenToNinePattern::Sc1Key3To9
        );
        assert_eq!(
            SevenToNinePattern::from_id(3),
            SevenToNinePattern::Sc2Key3To9
        );
        assert_eq!(
            SevenToNinePattern::from_id(4),
            SevenToNinePattern::Sc8Key1To7
        );
        assert_eq!(
            SevenToNinePattern::from_id(5),
            SevenToNinePattern::Sc9Key1To7
        );
        assert_eq!(
            SevenToNinePattern::from_id(6),
            SevenToNinePattern::Sc9Key2To8
        );
    }

    #[test]
    fn test_type_from_id() {
        assert_eq!(SevenToNineType::from_id(0), SevenToNineType::Passthrough);
        assert_eq!(SevenToNineType::from_id(1), SevenToNineType::AlternateAvoid);
        assert_eq!(SevenToNineType::from_id(2), SevenToNineType::Alternate);
        assert_eq!(SevenToNineType::from_id(99), SevenToNineType::Passthrough);
    }
}

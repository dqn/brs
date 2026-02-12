// Extra note modifier — takes notes from background and places on blank lanes.
//
// Ported from Java: ExtraNoteModifier.java

use std::collections::BTreeMap;

use bms_model::{BmsModel, Note, NoteType};

use crate::modifier::{AssistLevel, PatternModifier};

/// Takes notes from background and places them on blank lanes.
///
/// Java: `ExtraNoteModifier`
pub struct ExtraNoteModifier {
    /// Maximum number of notes to add per timeline
    pub depth: usize,
    /// Whether scratch lanes are included as placement targets
    pub include_scratch: bool,
    /// Track assist level
    assist: AssistLevel,
}

impl ExtraNoteModifier {
    pub fn new(depth: usize, include_scratch: bool) -> Self {
        Self {
            depth,
            include_scratch,
            assist: AssistLevel::None,
        }
    }
}

impl PatternModifier for ExtraNoteModifier {
    fn modify(&mut self, model: &mut BmsModel) {
        let key_count = model.mode.key_count();

        // Group note indices by time_us, sorted
        let mut time_groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (idx, note) in model.notes.iter().enumerate() {
            time_groups.entry(note.time_us).or_default().push(idx);
        }

        // Group bg_notes by time_us (as indices into model.bg_notes)
        let mut bg_time_groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (idx, bg) in model.bg_notes.iter().enumerate() {
            bg_time_groups.entry(bg.time_us).or_default().push(idx);
        }

        // Collect all unique times from both notes and bg_notes
        let mut all_times: Vec<i64> = time_groups
            .keys()
            .chain(bg_time_groups.keys())
            .copied()
            .collect();
        all_times.sort_unstable();
        all_times.dedup();

        // Track state
        let mut ln_active = vec![false; key_count];
        let mut last_note: Vec<Option<u16>> = vec![None; key_count]; // wav_id per lane
        let mut last_offset = 0usize;

        // Notes to add and bg_notes to remove
        let mut new_notes: Vec<Note> = Vec::new();
        let mut bg_indices_to_remove: Vec<usize> = Vec::new();

        for &time_us in &all_times {
            // Compute blank lanes
            let mut blank = vec![false; key_count];
            let note_indices = time_groups.get(&time_us);

            for key in 0..key_count {
                let has_note = note_indices
                    .map(|idxs| idxs.iter().any(|&i| model.notes[i].lane == key))
                    .unwrap_or(false);

                // Also check if we already added a note on this lane at this time
                let has_new_note = new_notes
                    .iter()
                    .any(|n| n.time_us == time_us && n.lane == key);

                if let Some(idxs) = note_indices {
                    for &idx in idxs {
                        let note = &model.notes[idx];
                        if note.lane == key && note.is_long_note() {
                            ln_active[key] = note.end_time_us > 0;
                        }
                    }
                }

                blank[key] = !ln_active[key]
                    && !has_note
                    && !has_new_note
                    && (self.include_scratch || !model.mode.is_scratch_key(key));
            }

            // Try to place bg notes on blank lanes
            for _d in 0..self.depth {
                let bg_indices = bg_time_groups.get(&time_us);
                let bg_idx = bg_indices.and_then(|idxs| {
                    idxs.iter()
                        .find(|&&i| !bg_indices_to_remove.contains(&i))
                        .copied()
                });

                if let Some(bi) = bg_idx {
                    let bg_note = &model.bg_notes[bi];
                    let wav_id = bg_note.wav_id;

                    // Find starting offset (try to match wav_id to last note)
                    let mut offset = last_offset;
                    for _j in 1..key_count {
                        offset = (offset + 1) % key_count;
                        if let Some(last_wav) = last_note[offset]
                            && last_wav == wav_id
                        {
                            break;
                        }
                    }
                    last_offset = offset;

                    // Find blank lane starting from offset
                    let mut placed = false;
                    for j in 0..key_count {
                        let key = (offset + j) % key_count;
                        if blank[key] {
                            last_note[key] = Some(wav_id);
                            new_notes.push(Note {
                                lane: key,
                                note_type: NoteType::Normal,
                                time_us,
                                end_time_us: 0,
                                wav_id,
                                end_wav_id: 0,
                                damage: 0,
                                pair_index: usize::MAX,
                                micro_starttime: bg_note.micro_starttime,
                                micro_duration: bg_note.micro_duration,
                            });
                            blank[key] = false;
                            bg_indices_to_remove.push(bi);
                            self.assist = AssistLevel::Assist;
                            placed = true;
                            break;
                        }
                    }

                    if !placed {
                        break; // No more blank lanes
                    }
                } else {
                    break; // No more bg notes at this time
                }
            }
        }

        // Add new notes to model
        model.notes.extend(new_notes);

        // Remove used bg_notes in reverse order
        bg_indices_to_remove.sort_unstable();
        bg_indices_to_remove.dedup();
        for &i in bg_indices_to_remove.iter().rev() {
            model.bg_notes.remove(i);
        }
    }

    fn assist_level(&self) -> AssistLevel {
        self.assist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::{BgNote, PlayMode};

    fn make_model(notes: Vec<Note>, bg_notes: Vec<BgNote>) -> BmsModel {
        BmsModel {
            mode: PlayMode::Beat7K,
            notes,
            bg_notes,
            ..Default::default()
        }
    }

    #[test]
    fn test_extra_note_places_bg_on_blank_lane() {
        let notes = vec![Note::normal(0, 1000, 1)];
        let bg_notes = vec![BgNote {
            wav_id: 10,
            time_us: 1000,
            micro_starttime: 0,
            micro_duration: 0,
        }];
        let mut model = make_model(notes, bg_notes);
        let mut modifier = ExtraNoteModifier::new(1, false);
        modifier.modify(&mut model);

        // bg note should be placed on a blank lane
        assert_eq!(model.notes.len(), 2);
        assert!(model.bg_notes.is_empty());
        assert_eq!(modifier.assist_level(), AssistLevel::Assist);

        // The new note should not be on lane 0 (occupied)
        let new_note = model.notes.iter().find(|n| n.wav_id == 10).unwrap();
        assert_ne!(new_note.lane, 0);
    }

    #[test]
    fn test_extra_note_no_bg_notes() {
        let notes = vec![Note::normal(0, 1000, 1)];
        let mut model = make_model(notes, Vec::new());
        let mut modifier = ExtraNoteModifier::new(1, false);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 1);
        assert_eq!(modifier.assist_level(), AssistLevel::None);
    }

    #[test]
    fn test_extra_note_depth_limits_additions() {
        let notes = vec![Note::normal(0, 1000, 1)];
        let bg_notes = vec![
            BgNote {
                wav_id: 10,
                time_us: 1000,
                micro_starttime: 0,
                micro_duration: 0,
            },
            BgNote {
                wav_id: 11,
                time_us: 1000,
                micro_starttime: 0,
                micro_duration: 0,
            },
            BgNote {
                wav_id: 12,
                time_us: 1000,
                micro_starttime: 0,
                micro_duration: 0,
            },
        ];
        let mut model = make_model(notes, bg_notes);
        let mut modifier = ExtraNoteModifier::new(2, false); // depth=2
        modifier.modify(&mut model);

        // Only 2 bg notes should be placed (depth=2)
        assert_eq!(model.notes.len(), 3); // 1 original + 2 added
        assert_eq!(model.bg_notes.len(), 1); // 1 remaining
    }

    #[test]
    fn test_extra_note_skips_scratch_lane() {
        // All lanes except scratch (lane 7) are occupied
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(1, 1000, 2),
            Note::normal(2, 1000, 3),
            Note::normal(3, 1000, 4),
            Note::normal(4, 1000, 5),
            Note::normal(5, 1000, 6),
            Note::normal(6, 1000, 7),
        ];
        let bg_notes = vec![BgNote {
            wav_id: 10,
            time_us: 1000,
            micro_starttime: 0,
            micro_duration: 0,
        }];
        let mut model = make_model(notes, bg_notes);
        let mut modifier = ExtraNoteModifier::new(1, false); // no scratch
        modifier.modify(&mut model);

        // Cannot place on scratch lane, so bg note remains
        assert_eq!(model.notes.len(), 7);
        assert_eq!(model.bg_notes.len(), 1);
    }

    #[test]
    fn test_extra_note_includes_scratch_lane() {
        // All lanes except scratch (lane 7) are occupied
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(1, 1000, 2),
            Note::normal(2, 1000, 3),
            Note::normal(3, 1000, 4),
            Note::normal(4, 1000, 5),
            Note::normal(5, 1000, 6),
            Note::normal(6, 1000, 7),
        ];
        let bg_notes = vec![BgNote {
            wav_id: 10,
            time_us: 1000,
            micro_starttime: 0,
            micro_duration: 0,
        }];
        let mut model = make_model(notes, bg_notes);
        let mut modifier = ExtraNoteModifier::new(1, true); // include scratch
        modifier.modify(&mut model);

        // Should place on scratch lane 7
        assert_eq!(model.notes.len(), 8);
        assert!(model.bg_notes.is_empty());
        let new_note = model.notes.iter().find(|n| n.wav_id == 10).unwrap();
        assert_eq!(new_note.lane, 7);
    }

    #[test]
    fn test_extra_note_all_lanes_full() {
        // All 8 lanes occupied
        let notes: Vec<Note> = (0..8)
            .map(|i| Note::normal(i, 1000, i as u16 + 1))
            .collect();
        let bg_notes = vec![BgNote {
            wav_id: 10,
            time_us: 1000,
            micro_starttime: 0,
            micro_duration: 0,
        }];
        let mut model = make_model(notes, bg_notes);
        let mut modifier = ExtraNoteModifier::new(1, true);
        modifier.modify(&mut model);

        // No blank lanes, bg note remains
        assert_eq!(model.notes.len(), 8);
        assert_eq!(model.bg_notes.len(), 1);
    }
}

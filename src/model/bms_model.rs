use crate::audio::BgmEvent;
use crate::model::note::{Lane, Note, NoteType};
use crate::model::timeline::{Timeline, Timelines};
use anyhow::Result;
use bms_rs::bms::command::channel::NoteKind;
use bms_rs::bms::command::channel::mapper::KeyLayoutBeat;
use bms_rs::bms::command::time::ObjTime;
use bms_rs::bms::prelude::*;
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use std::num::NonZeroU64;

/// Play mode for the BMS chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Beat5K,
    Beat7K,
    Beat10K,
    Beat14K,
    PopN5K,
    PopN9K,
}

/// Parsed BMS model with all game-relevant data.
#[derive(Debug, Clone)]
pub struct BMSModel {
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub initial_bpm: f64,
    pub total_notes: usize,
    pub play_mode: PlayMode,
    pub timelines: Timelines,
    pub wav_files: BTreeMap<u16, String>,
    /// BGM events (auto-play keysounds) sorted by time.
    pub bgm_events: Vec<BgmEvent>,
}

impl BMSModel {
    /// Convert from bms-rs Bms output.
    pub fn from_bms(bms: &Bms) -> Result<Self> {
        let title = bms
            .music_info
            .title
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let artist = bms
            .music_info
            .artist
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let genre = bms
            .music_info
            .genre
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        let initial_bpm = bms
            .bpm
            .bpm
            .as_ref()
            .and_then(|d| d.to_f64())
            .unwrap_or(120.0);

        let time_calc = TimeCalculator::new(bms, initial_bpm);
        let (mut timelines, bgm_events) = build_timelines(bms, &time_calc)?;

        let total_notes = timelines
            .all_notes()
            .filter(|n| {
                matches!(n.note_type, NoteType::Normal | NoteType::LongStart)
                    && n.note_type != NoteType::Invisible
            })
            .count();

        timelines.sort_by_time();

        let wav_files = bms
            .wav
            .wav_files
            .iter()
            .filter_map(|(id, path)| {
                let id_num = obj_id_to_u16(*id)?;
                Some((id_num, path.to_string_lossy().to_string()))
            })
            .collect();

        Ok(Self {
            title,
            artist,
            genre,
            initial_bpm,
            total_notes,
            play_mode: PlayMode::Beat7K,
            timelines,
            wav_files,
            bgm_events,
        })
    }
}

/// Helper to calculate time from ObjTime considering BPM and section length changes.
struct TimeCalculator {
    initial_bpm: f64,
    bpm_changes: Vec<(f64, f64)>,
    section_lengths: BTreeMap<u32, f64>,
    measure_start_times: BTreeMap<u64, f64>,
}

impl TimeCalculator {
    fn new(bms: &Bms, initial_bpm: f64) -> Self {
        let section_lengths: BTreeMap<u32, f64> = bms
            .section_len
            .section_len_changes
            .iter()
            .map(|(track, change)| (track.0 as u32, change.length.to_f64().unwrap_or(1.0)))
            .collect();

        let mut bpm_changes: Vec<(f64, f64)> = Vec::new();
        let mut measure_start_times: BTreeMap<u64, f64> = BTreeMap::new();

        let max_measure = bms
            .notes()
            .all_notes()
            .map(|n| n.offset.track().0)
            .max()
            .unwrap_or(1);

        let mut current_time = 0.0;
        let mut current_bpm = initial_bpm;

        for measure in 0..=max_measure + 1 {
            measure_start_times.insert(measure, current_time);

            let section_len = Self::get_section_length_at(&section_lengths, measure);

            let bpm_changes_in_measure: Vec<_> = bms
                .bpm
                .bpm_changes
                .iter()
                .filter(|(time, _)| time.track().0 == measure)
                .map(|(time, change)| {
                    let pos = time.numerator() as f64 / time.denominator_u64() as f64;
                    let bpm = change.bpm.to_f64().unwrap_or(initial_bpm);
                    (pos, bpm)
                })
                .collect();

            let mut positions: Vec<f64> = bpm_changes_in_measure.iter().map(|(p, _)| *p).collect();
            positions.push(1.0);
            positions.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
            positions.dedup();

            let mut last_pos = 0.0;
            for pos in positions {
                if pos > last_pos {
                    let duration = Self::calc_duration(pos - last_pos, section_len, current_bpm);
                    current_time += duration;
                }

                for (change_pos, new_bpm) in &bpm_changes_in_measure {
                    if (*change_pos - pos).abs() < 1e-9 {
                        bpm_changes.push((current_time, *new_bpm));
                        current_bpm = *new_bpm;
                    }
                }

                last_pos = pos;
            }
        }

        Self {
            initial_bpm,
            bpm_changes,
            section_lengths,
            measure_start_times,
        }
    }

    fn get_section_length_at(section_lengths: &BTreeMap<u32, f64>, measure: u64) -> f64 {
        section_lengths
            .range(..=(measure as u32))
            .last()
            .map(|(_, len)| *len)
            .unwrap_or(1.0)
    }

    fn calc_duration(fraction: f64, section_len: f64, bpm: f64) -> f64 {
        let quarter_note_ms = 60000.0 / bpm;
        let measure_ms = quarter_note_ms * 4.0 * section_len;
        measure_ms * fraction
    }

    fn objtime_to_ms(&self, time: &ObjTime) -> f64 {
        let measure = time.track().0;
        let pos = time.numerator() as f64 / time.denominator_u64() as f64;

        let measure_start = self
            .measure_start_times
            .get(&measure)
            .copied()
            .unwrap_or(0.0);

        let bpm_at_measure_start = self
            .bpm_changes
            .iter()
            .rev()
            .find(|(t, _)| *t <= measure_start)
            .map(|(_, bpm)| *bpm)
            .unwrap_or(self.initial_bpm);

        let section_len = Self::get_section_length_at(&self.section_lengths, measure);

        let mut current_time = measure_start;
        let mut current_bpm = bpm_at_measure_start;
        let mut last_pos = 0.0;

        let mut bpm_changes_in_measure: Vec<(f64, f64)> = self
            .bpm_changes
            .iter()
            .filter(|(t, _)| {
                *t >= measure_start
                    && *t
                        < self
                            .measure_start_times
                            .get(&(measure + 1))
                            .copied()
                            .unwrap_or(f64::MAX)
            })
            .map(|(_, bpm)| (0.0, *bpm))
            .collect();

        for (change_pos, new_bpm) in &mut bpm_changes_in_measure {
            if *change_pos < pos {
                let duration =
                    Self::calc_duration(*change_pos - last_pos, section_len, current_bpm);
                current_time += duration;
                current_bpm = *new_bpm;
                last_pos = *change_pos;
            }
        }

        let remaining = pos - last_pos;
        if remaining > 0.0 {
            current_time += Self::calc_duration(remaining, section_len, current_bpm);
        }

        current_time
    }
}

fn build_timelines(bms: &Bms, time_calc: &TimeCalculator) -> Result<(Timelines, Vec<BgmEvent>)> {
    let mut timelines = Timelines::new();
    let mut bgm_events = Vec::new();

    let max_measure = bms
        .notes()
        .all_notes()
        .map(|n| n.offset.track().0)
        .max()
        .unwrap_or(1);

    for measure in 0..=max_measure {
        let time = ObjTime::new(measure, 0, NonZeroU64::new(1).unwrap());
        let time_ms = time_calc.objtime_to_ms(&time);

        let bpm = time_calc
            .bpm_changes
            .iter()
            .rev()
            .find(|(t, _)| *t <= time_ms)
            .map(|(_, b)| *b)
            .unwrap_or(time_calc.initial_bpm);

        timelines.push(Timeline::measure_line(time_ms, measure as u32, bpm));
    }

    let mut long_note_starts: BTreeMap<Lane, (f64, u16)> = BTreeMap::new();

    for note in bms.notes().all_notes() {
        if note.wav_id.is_null() {
            continue;
        }

        let time_ms = time_calc.objtime_to_ms(&note.offset);
        let wav_id = obj_id_to_u16(note.wav_id).unwrap_or(0);

        let lane = channel_to_lane(&note.channel_id);

        // Notes without a lane mapping are BGM (auto-play) keysounds
        if lane.is_none() {
            bgm_events.push(BgmEvent::new(time_ms, wav_id));
            continue;
        }
        let lane = lane.unwrap();

        let note_kind = get_note_kind(&note.channel_id);

        let note_obj = match note_kind {
            Some(NoteKind::Visible) => Note::normal(lane, time_ms, wav_id),
            Some(NoteKind::Invisible) => Note::invisible(lane, time_ms, wav_id),
            Some(NoteKind::Long) => {
                if let Some((start_ms, start_wav_id)) = long_note_starts.remove(&lane) {
                    Note::long_start(lane, start_ms, time_ms, start_wav_id)
                } else {
                    long_note_starts.insert(lane, (time_ms, wav_id));
                    continue;
                }
            }
            _ => continue,
        };

        let measure = note.offset.track().0;
        let pos = note.offset.numerator() as f64 / note.offset.denominator_u64() as f64;

        let bpm = time_calc
            .bpm_changes
            .iter()
            .rev()
            .find(|(t, _)| *t <= time_ms)
            .map(|(_, b)| *b)
            .unwrap_or(time_calc.initial_bpm);

        let mut timeline = Timeline::new(time_ms, measure as u32, pos, bpm);
        timeline.add_note(note_obj);
        timelines.push(timeline);
    }

    // Sort BGM events by time
    bgm_events.sort_by(|a, b| {
        a.time_ms
            .partial_cmp(&b.time_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok((timelines, bgm_events))
}

fn channel_to_lane(channel_id: &bms_rs::bms::command::channel::NoteChannelId) -> Option<Lane> {
    use bms_rs::bms::command::channel::mapper::KeyMapping;
    use bms_rs::bms::prelude::Key;

    let layout = KeyLayoutBeat::from_channel_id(*channel_id)?;
    let key = layout.key();

    match key {
        Key::Scratch(_) => Some(Lane::Scratch),
        Key::Key(1) => Some(Lane::Key1),
        Key::Key(2) => Some(Lane::Key2),
        Key::Key(3) => Some(Lane::Key3),
        Key::Key(4) => Some(Lane::Key4),
        Key::Key(5) => Some(Lane::Key5),
        Key::Key(6) => Some(Lane::Key6),
        Key::Key(7) => Some(Lane::Key7),
        _ => None,
    }
}

fn get_note_kind(channel_id: &bms_rs::bms::command::channel::NoteChannelId) -> Option<NoteKind> {
    use bms_rs::bms::command::channel::mapper::KeyMapping;

    let layout = KeyLayoutBeat::from_channel_id(*channel_id)?;
    Some(layout.kind())
}

fn obj_id_to_u16(id: bms_rs::bms::command::ObjId) -> Option<u16> {
    let s = id.to_string();
    if s.len() >= 2 {
        u16::from_str_radix(&s, 36).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bms_loader::load_bms;
    use std::path::Path;

    #[test]
    fn test_bms_model_from_bms() {
        let path = Path::new("bms/bms-002/_take_7N.bms");
        if path.exists() {
            let bms = load_bms(path).expect("Failed to load BMS");
            let model = BMSModel::from_bms(&bms).expect("Failed to create model");

            println!("Title: {}", model.title);
            println!("Artist: {}", model.artist);
            println!("BPM: {}", model.initial_bpm);
            println!("Total notes: {}", model.total_notes);

            assert!(!model.title.is_empty());
            assert!(model.initial_bpm > 0.0);
        }
    }
}

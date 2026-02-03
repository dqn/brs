use crate::audio::BgmEvent;
use crate::model::note::{Lane, Note, NoteType};
use crate::model::timing::TimingEngine;
use crate::model::timeline::{Timeline, Timelines};
use anyhow::Result;
use bms_rs::bms::command::channel::NoteKind;
use bms_rs::bms::command::channel::mapper::KeyLayoutBeat;
use bms_rs::bms::command::time::ObjTime;
use bms_rs::bms::prelude::*;
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

        let timing = TimingEngine::new(bms);
        let initial_bpm = timing.initial_bpm();
        let (mut timelines, bgm_events) = build_timelines(bms, &timing)?;

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

fn build_timelines(bms: &Bms, timing: &TimingEngine) -> Result<(Timelines, Vec<BgmEvent>)> {
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
        let time_ms = timing.objtime_to_ms(time);
        let bpm = timing.bpm_at(time);

        timelines.push(Timeline::measure_line(time_ms, measure as u32, bpm));
    }

    let mut long_note_starts: BTreeMap<Lane, (f64, u16)> = BTreeMap::new();

    for note in bms.notes().all_notes() {
        if note.wav_id.is_null() {
            continue;
        }

        let time_ms = timing.objtime_to_ms(note.offset);
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

        let bpm = timing.bpm_at(note.offset);

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

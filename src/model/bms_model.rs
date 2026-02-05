use std::collections::BTreeMap;
use std::num::NonZeroU64;

use anyhow::Result;
use bms_rs::bms::command::channel::NoteKind;
use bms_rs::bms::command::channel::mapper::{KeyLayoutBeat, KeyLayoutPms};
use bms_rs::bms::command::time::ObjTime;
use bms_rs::bms::prelude::*;
use num_traits::ToPrimitive;

use crate::audio::BgmEvent;
use crate::model::note::{Lane, Note, NoteType};
use crate::model::timeline::{Timeline, Timelines};
use crate::model::timing::TimingEngine;
use crate::model::{BgaEvent, BgaLayer, ChartFormat};

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

/// Source of the judge rank definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeRankType {
    BmsRank,
    BmsDefExRank,
    BmsonJudgeRank,
}

/// Source of the TOTAL value definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotalType {
    Bms,
    Bmson,
}

/// Long note mode used for judgement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongNoteMode {
    Ln,
    Cn,
    Hcn,
}

impl From<LnMode> for LongNoteMode {
    fn from(mode: LnMode) -> Self {
        match mode {
            LnMode::Ln => LongNoteMode::Ln,
            LnMode::Cn => LongNoteMode::Cn,
            LnMode::Hcn => LongNoteMode::Hcn,
        }
    }
}

/// Parsed BMS model with all game-relevant data.
#[derive(Debug, Clone)]
pub struct BMSModel {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub subartist: String,
    pub genre: String,
    pub preview: Option<String>,
    pub stage_file: Option<String>,
    pub back_bmp: Option<String>,
    pub banner: Option<String>,
    pub initial_bpm: f64,
    pub min_bpm: f64,
    pub max_bpm: f64,
    pub total_notes: usize,
    pub total: f64,
    pub total_type: TotalType,
    pub judge_rank: i32,
    pub judge_rank_type: JudgeRankType,
    pub long_note_mode: LongNoteMode,
    pub play_mode: PlayMode,
    pub source_format: ChartFormat,
    pub has_long_note: bool,
    pub has_mine: bool,
    pub has_invisible: bool,
    pub has_stop: bool,
    pub play_level: Option<u8>,
    pub difficulty: Option<u8>,
    pub folder: String,
    pub timelines: Timelines,
    pub wav_files: BTreeMap<u16, String>,
    pub bga_files: BTreeMap<u16, String>,
    pub bga_events: Vec<BgaEvent>,
    pub poor_bga_file: Option<String>,
    /// BGM events (auto-play keysounds) sorted by time.
    pub bgm_events: Vec<BgmEvent>,
}

impl BMSModel {
    /// Convert from bms-rs Bms output.
    pub fn from_bms(
        bms: &Bms,
        format: ChartFormat,
        source_path: Option<&std::path::Path>,
    ) -> Result<Self> {
        let title = bms
            .music_info
            .title
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let subtitle = bms.music_info.subtitle.clone().unwrap_or_default();
        let artist = bms
            .music_info
            .artist
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let subartist = bms.music_info.sub_artist.clone().unwrap_or_default();
        let genre = bms
            .music_info
            .genre
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let preview = bms
            .music_info
            .preview_music
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let stage_file = bms
            .sprite
            .stage_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let back_bmp = bms
            .sprite
            .back_bmp
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let banner = bms
            .sprite
            .banner
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let play_level = bms.metadata.play_level;
        let difficulty = bms.metadata.difficulty;
        let folder = source_path
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();

        let play_mode = detect_mode(bms, source_path);

        let timing = TimingEngine::new(bms);
        let initial_bpm = timing.initial_bpm();
        let (mut timelines, bgm_events) = build_timelines(bms, &timing, play_mode)?;

        let long_note_mode = LongNoteMode::from(bms.repr.ln_mode);
        let total_notes = count_total_notes(&timelines, long_note_mode);
        let has_long_note = timelines
            .all_notes()
            .any(|n| n.note_type == NoteType::LongStart);
        let has_invisible = timelines
            .all_notes()
            .any(|n| n.note_type == NoteType::Invisible);
        let has_mine = timelines.all_notes().any(|n| n.note_type == NoteType::Mine);
        let has_stop = !bms.stop.stops.is_empty();
        let (min_bpm, max_bpm) = calc_bpm_range(bms, initial_bpm);

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

        let (bga_files, bga_events, poor_bga_file) = build_bga_data(bms, &timing);

        let (judge_rank, judge_rank_type) = extract_judge_rank(bms, format);
        let (total, total_type) = extract_total(bms, format);

        Ok(Self {
            title,
            subtitle,
            artist,
            subartist,
            genre,
            preview,
            stage_file,
            back_bmp,
            banner,
            initial_bpm,
            min_bpm,
            max_bpm,
            total_notes,
            total,
            total_type,
            judge_rank,
            judge_rank_type,
            long_note_mode,
            play_mode,
            source_format: format,
            has_long_note,
            has_mine,
            has_invisible,
            has_stop,
            play_level,
            difficulty,
            folder,
            timelines,
            wav_files,
            bga_files,
            bga_events,
            poor_bga_file,
            bgm_events,
        })
    }
}

#[derive(Debug, Clone)]
struct PendingLongNote {
    timeline_index: usize,
    note_index: usize,
}

fn build_timelines(
    bms: &Bms,
    timing: &TimingEngine,
    play_mode: PlayMode,
) -> Result<(Timelines, Vec<BgmEvent>)> {
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

    let mut long_note_starts: BTreeMap<Lane, PendingLongNote> = BTreeMap::new();

    fn push_note(
        timelines: &mut Timelines,
        note: Note,
        measure: u32,
        pos: f64,
        bpm: f64,
    ) -> (usize, usize) {
        let mut timeline = Timeline::new(note.start_time_ms, measure, pos, bpm);
        timeline.add_note(note);
        timelines.push(timeline);
        let timeline_index = timelines.entries().len() - 1;
        (timeline_index, 0)
    }

    for note in bms.notes().all_notes() {
        if note.wav_id.is_null() {
            continue;
        }

        let time_ms = timing.objtime_to_ms(note.offset);
        let wav_id = obj_id_to_u16(note.wav_id).unwrap_or(0);

        let lane = channel_to_lane(&note.channel_id, play_mode);

        // Notes without a lane mapping are BGM (auto-play) keysounds
        if lane.is_none() {
            bgm_events.push(BgmEvent::new(time_ms, wav_id));
            continue;
        }
        let lane = lane.unwrap();

        let note_kind = get_note_kind(&note.channel_id, play_mode);

        let measure = note.offset.track().0;
        let pos = note.offset.numerator() as f64 / note.offset.denominator_u64() as f64;
        let bpm = timing.bpm_at(note.offset);

        match note_kind {
            Some(NoteKind::Visible) => {
                let note_obj = Note::normal(lane, time_ms, wav_id);
                push_note(&mut timelines, note_obj, measure as u32, pos, bpm);
            }
            Some(NoteKind::Invisible) => {
                let note_obj = Note::invisible(lane, time_ms, wav_id);
                push_note(&mut timelines, note_obj, measure as u32, pos, bpm);
            }
            Some(NoteKind::Landmine) => {
                let damage = wav_id as f64;
                let note_obj = Note::mine(lane, time_ms, damage);
                push_note(&mut timelines, note_obj, measure as u32, pos, bpm);
            }
            Some(NoteKind::Long) => {
                if let Some(pending) = long_note_starts.remove(&lane) {
                    if let Some(start_timeline) =
                        timelines.entries_mut().get_mut(pending.timeline_index)
                    {
                        if let Some(start_note) = start_timeline.notes.get_mut(pending.note_index) {
                            start_note.end_time_ms = Some(time_ms);
                        }
                    }

                    let note_obj = Note::long_end(lane, time_ms, wav_id);
                    push_note(&mut timelines, note_obj, measure as u32, pos, bpm);
                } else {
                    let note_obj = Note {
                        lane,
                        start_time_ms: time_ms,
                        end_time_ms: None,
                        wav_id,
                        note_type: NoteType::LongStart,
                        mine_damage: None,
                    };
                    let (timeline_index, note_index) =
                        push_note(&mut timelines, note_obj, measure as u32, pos, bpm);
                    long_note_starts.insert(
                        lane,
                        PendingLongNote {
                            timeline_index,
                            note_index,
                        },
                    );
                }
            }
            _ => {}
        }
    }

    // Sort BGM events by time
    bgm_events.sort_by(|a, b| {
        a.time_ms
            .partial_cmp(&b.time_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok((timelines, bgm_events))
}

fn channel_to_lane(
    channel_id: &bms_rs::bms::command::channel::NoteChannelId,
    play_mode: PlayMode,
) -> Option<Lane> {
    use bms_rs::bms::command::channel::mapper::KeyMapping;
    use bms_rs::bms::prelude::{Key, PlayerSide};

    match play_mode {
        PlayMode::PopN5K | PlayMode::PopN9K => {
            let layout = KeyLayoutPms::from_channel_id(*channel_id)?;
            let key = layout.key();
            let lane = match key {
                Key::Key(1) => Some(Lane::Key1),
                Key::Key(2) => Some(Lane::Key2),
                Key::Key(3) => Some(Lane::Key3),
                Key::Key(4) => Some(Lane::Key4),
                Key::Key(5) => Some(Lane::Key5),
                Key::Key(6) => Some(Lane::Key6),
                Key::Key(7) => Some(Lane::Key7),
                Key::Key(8) => Some(Lane::Key8),
                Key::Key(9) => Some(Lane::Key9),
                _ => None,
            };

            match play_mode {
                PlayMode::PopN5K => match lane {
                    Some(Lane::Key1 | Lane::Key2 | Lane::Key3 | Lane::Key4 | Lane::Key5) => lane,
                    _ => None,
                },
                PlayMode::PopN9K => match lane {
                    Some(
                        Lane::Key1
                        | Lane::Key2
                        | Lane::Key3
                        | Lane::Key4
                        | Lane::Key5
                        | Lane::Key6
                        | Lane::Key7
                        | Lane::Key8
                        | Lane::Key9,
                    ) => lane,
                    _ => None,
                },
                _ => None,
            }
        }
        PlayMode::Beat5K | PlayMode::Beat7K | PlayMode::Beat10K | PlayMode::Beat14K => {
            let layout = KeyLayoutBeat::from_channel_id(*channel_id)?;
            let key = layout.key();
            let side = layout.side();

            match play_mode {
                PlayMode::Beat5K => match (side, key) {
                    (PlayerSide::Player1, Key::Scratch(_)) => Some(Lane::Scratch),
                    (PlayerSide::Player1, Key::Key(1)) => Some(Lane::Key1),
                    (PlayerSide::Player1, Key::Key(2)) => Some(Lane::Key2),
                    (PlayerSide::Player1, Key::Key(3)) => Some(Lane::Key3),
                    (PlayerSide::Player1, Key::Key(4)) => Some(Lane::Key4),
                    (PlayerSide::Player1, Key::Key(5)) => Some(Lane::Key5),
                    _ => None,
                },
                PlayMode::Beat7K => match (side, key) {
                    (PlayerSide::Player1, Key::Scratch(_)) => Some(Lane::Scratch),
                    (PlayerSide::Player1, Key::Key(1)) => Some(Lane::Key1),
                    (PlayerSide::Player1, Key::Key(2)) => Some(Lane::Key2),
                    (PlayerSide::Player1, Key::Key(3)) => Some(Lane::Key3),
                    (PlayerSide::Player1, Key::Key(4)) => Some(Lane::Key4),
                    (PlayerSide::Player1, Key::Key(5)) => Some(Lane::Key5),
                    (PlayerSide::Player1, Key::Key(6)) => Some(Lane::Key6),
                    (PlayerSide::Player1, Key::Key(7)) => Some(Lane::Key7),
                    _ => None,
                },
                PlayMode::Beat10K => match (side, key) {
                    (PlayerSide::Player1, Key::Scratch(_)) => Some(Lane::Scratch),
                    (PlayerSide::Player1, Key::Key(1)) => Some(Lane::Key1),
                    (PlayerSide::Player1, Key::Key(2)) => Some(Lane::Key2),
                    (PlayerSide::Player1, Key::Key(3)) => Some(Lane::Key3),
                    (PlayerSide::Player1, Key::Key(4)) => Some(Lane::Key4),
                    (PlayerSide::Player1, Key::Key(5)) => Some(Lane::Key5),
                    (PlayerSide::Player2, Key::Scratch(_)) => Some(Lane::Scratch2),
                    (PlayerSide::Player2, Key::Key(1)) => Some(Lane::Key8),
                    (PlayerSide::Player2, Key::Key(2)) => Some(Lane::Key9),
                    (PlayerSide::Player2, Key::Key(3)) => Some(Lane::Key10),
                    (PlayerSide::Player2, Key::Key(4)) => Some(Lane::Key11),
                    (PlayerSide::Player2, Key::Key(5)) => Some(Lane::Key12),
                    _ => None,
                },
                PlayMode::Beat14K => match (side, key) {
                    (PlayerSide::Player1, Key::Scratch(_)) => Some(Lane::Scratch),
                    (PlayerSide::Player1, Key::Key(1)) => Some(Lane::Key1),
                    (PlayerSide::Player1, Key::Key(2)) => Some(Lane::Key2),
                    (PlayerSide::Player1, Key::Key(3)) => Some(Lane::Key3),
                    (PlayerSide::Player1, Key::Key(4)) => Some(Lane::Key4),
                    (PlayerSide::Player1, Key::Key(5)) => Some(Lane::Key5),
                    (PlayerSide::Player1, Key::Key(6)) => Some(Lane::Key6),
                    (PlayerSide::Player1, Key::Key(7)) => Some(Lane::Key7),
                    (PlayerSide::Player2, Key::Scratch(_)) => Some(Lane::Scratch2),
                    (PlayerSide::Player2, Key::Key(1)) => Some(Lane::Key8),
                    (PlayerSide::Player2, Key::Key(2)) => Some(Lane::Key9),
                    (PlayerSide::Player2, Key::Key(3)) => Some(Lane::Key10),
                    (PlayerSide::Player2, Key::Key(4)) => Some(Lane::Key11),
                    (PlayerSide::Player2, Key::Key(5)) => Some(Lane::Key12),
                    (PlayerSide::Player2, Key::Key(6)) => Some(Lane::Key13),
                    (PlayerSide::Player2, Key::Key(7)) => Some(Lane::Key14),
                    _ => None,
                },
                _ => None,
            }
        }
    }
}

fn get_note_kind(
    channel_id: &bms_rs::bms::command::channel::NoteChannelId,
    play_mode: PlayMode,
) -> Option<NoteKind> {
    use bms_rs::bms::command::channel::mapper::KeyMapping;

    match play_mode {
        PlayMode::PopN5K | PlayMode::PopN9K => {
            let layout = KeyLayoutPms::from_channel_id(*channel_id)?;
            Some(layout.kind())
        }
        _ => {
            let layout = KeyLayoutBeat::from_channel_id(*channel_id)?;
            Some(layout.kind())
        }
    }
}

fn obj_id_to_u16(id: bms_rs::bms::command::ObjId) -> Option<u16> {
    let s = id.to_string();
    if s.len() >= 2 {
        u16::from_str_radix(&s, 36).ok()
    } else {
        None
    }
}

fn count_total_notes(timelines: &Timelines, long_note_mode: LongNoteMode) -> usize {
    timelines
        .all_notes()
        .filter(|note| match note.note_type {
            NoteType::Normal => true,
            NoteType::LongStart => true,
            NoteType::LongEnd => !matches!(long_note_mode, LongNoteMode::Ln),
            _ => false,
        })
        .count()
}

fn calc_bpm_range(bms: &Bms, initial_bpm: f64) -> (f64, f64) {
    let mut min_bpm = initial_bpm;
    let mut max_bpm = initial_bpm;

    for change in bms.bpm.bpm_changes.values() {
        if let Some(bpm) = change.bpm.to_f64() {
            if bpm > 0.0 {
                min_bpm = min_bpm.min(bpm);
                max_bpm = max_bpm.max(bpm);
            }
        }
    }

    for bpm in bms.bpm.bpm_changes_u8.values() {
        let bpm = *bpm as f64;
        if bpm > 0.0 {
            min_bpm = min_bpm.min(bpm);
            max_bpm = max_bpm.max(bpm);
        }
    }

    (min_bpm, max_bpm)
}

fn build_bga_data(
    bms: &Bms,
    timing: &TimingEngine,
) -> (BTreeMap<u16, String>, Vec<BgaEvent>, Option<String>) {
    use bms_rs::bms::model::obj::BgaLayer as SourceBgaLayer;

    let mut bga_files = BTreeMap::new();
    for (id, bmp) in &bms.bmp.bmp_files {
        if let Some(id) = obj_id_to_u16(*id) {
            bga_files.insert(id, bmp.file.to_string_lossy().to_string());
        }
    }

    let mut bga_events = Vec::new();
    for (time, bga_obj) in &bms.bmp.bga_changes {
        let Some(id) = obj_id_to_u16(bga_obj.id) else {
            continue;
        };
        let time_ms = timing.objtime_to_ms(*time);
        let layer = match bga_obj.layer {
            SourceBgaLayer::Base => BgaLayer::Base,
            SourceBgaLayer::Overlay => BgaLayer::Layer,
            SourceBgaLayer::Overlay2 => BgaLayer::Layer2,
            SourceBgaLayer::Poor => BgaLayer::Poor,
            _ => continue,
        };
        bga_events.push(BgaEvent {
            time_ms,
            bga_id: id,
            layer,
        });
    }

    let poor_bga_file = bms
        .bmp
        .poor_bmp
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());

    (bga_files, bga_events, poor_bga_file)
}

fn extract_judge_rank(bms: &Bms, format: ChartFormat) -> (i32, JudgeRankType) {
    fn judge_level_to_i32(level: JudgeLevel) -> i32 {
        match level {
            JudgeLevel::VeryHard => 0,
            JudgeLevel::Hard => 1,
            JudgeLevel::Normal => 2,
            JudgeLevel::Easy => 3,
            JudgeLevel::OtherInt(value) => value as i32,
        }
    }

    match format {
        ChartFormat::Bmson => {
            if let Some(rank) = bms.judge.rank {
                return (judge_level_to_i32(rank), JudgeRankType::BmsonJudgeRank);
            }
            (2, JudgeRankType::BmsonJudgeRank)
        }
        ChartFormat::Bms => {
            if let Some(rank) = bms.judge.rank {
                return match rank {
                    JudgeLevel::OtherInt(value) => (value as i32, JudgeRankType::BmsDefExRank),
                    _ => (judge_level_to_i32(rank), JudgeRankType::BmsRank),
                };
            }

            if let Ok(def_id) = bms_rs::bms::command::ObjId::try_from("00", false) {
                if let Some(def) = bms.judge.exrank_defs.get(&def_id) {
                    return match def.judge_level {
                        JudgeLevel::OtherInt(value) => (value as i32, JudgeRankType::BmsDefExRank),
                        _ => (
                            judge_level_to_i32(def.judge_level),
                            JudgeRankType::BmsDefExRank,
                        ),
                    };
                }
            }

            (2, JudgeRankType::BmsRank)
        }
    }
}

fn extract_total(bms: &Bms, format: ChartFormat) -> (f64, TotalType) {
    let total = bms
        .judge
        .total
        .as_ref()
        .and_then(|t| t.to_f64())
        .unwrap_or(200.0);

    let total_type = match format {
        ChartFormat::Bms => TotalType::Bms,
        ChartFormat::Bmson => TotalType::Bmson,
    };

    (total, total_type)
}

fn detect_mode(bms: &Bms, source_path: Option<&std::path::Path>) -> PlayMode {
    let ext = source_path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    if matches!(ext.as_deref(), Some("pms")) {
        let mut max_key = 0;
        for note in bms.notes().all_notes() {
            if let Some(layout) = KeyLayoutPms::from_channel_id(note.channel_id) {
                if let bms_rs::bms::prelude::Key::Key(key) = layout.key() {
                    max_key = max_key.max(key);
                }
            }
        }
        return if max_key >= 9 {
            PlayMode::PopN9K
        } else {
            PlayMode::PopN5K
        };
    }

    let mut has_player2 = false;
    let mut max_key = 0;
    let mut max_key_p2 = 0;

    for note in bms.notes().all_notes() {
        if let Some(layout) = KeyLayoutBeat::from_channel_id(note.channel_id) {
            if layout.side() == bms_rs::bms::prelude::PlayerSide::Player2 {
                has_player2 = true;
            }
            if let bms_rs::bms::prelude::Key::Key(key) = layout.key() {
                max_key = max_key.max(key);
                if layout.side() == bms_rs::bms::prelude::PlayerSide::Player2 {
                    max_key_p2 = max_key_p2.max(key);
                }
            }
        }
    }

    if has_player2 {
        if max_key >= 7 || max_key_p2 >= 7 {
            PlayMode::Beat14K
        } else {
            PlayMode::Beat10K
        }
    } else if max_key >= 7 {
        PlayMode::Beat7K
    } else {
        PlayMode::Beat5K
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::model::bms_loader::load_chart;

    #[test]
    fn test_bms_model_from_bms() {
        let path = Path::new("bms/bms-002/_take_7N.bms");
        if path.exists() {
            let loaded = load_chart(path).expect("Failed to load BMS");
            let model = BMSModel::from_bms(&loaded.bms, loaded.format, Some(path))
                .expect("Failed to create model");

            println!("Title: {}", model.title);
            println!("Artist: {}", model.artist);
            println!("BPM: {}", model.initial_bpm);
            println!("Total notes: {}", model.total_notes);

            assert!(!model.title.is_empty());
            assert!(model.initial_bpm > 0.0);
        }
    }
}

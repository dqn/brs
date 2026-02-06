// BMS loader: converts bms-rs parsed data into internal BMSModel.

use anyhow::Result;
use bms_rs::bms::model::Bms;
use bms_rs::bms::{default_config, parse_bms};
use bms_rs::command::channel::mapper::{KeyLayoutBeat, KeyLayoutMapper, KeyMapping};
use bms_rs::command::channel::{Key, NoteKind, PlayerSide};
use bms_rs::command::{JudgeLevel, PlayerMode};
use num_traits::ToPrimitive;
use std::path::Path;

use super::bms_model::{BgaEvent, BgmEvent, BmsModel, BpmChange, StopEvent, WavDef};
use super::note::{JudgeRankType, LongNoteMode, Note, NoteType, PlayMode};
use super::timing::{BpmEvent, StopSpec, TimingEngine};

/// Load a BMS/BME/BML file from the given path.
pub fn load_bms(path: &Path) -> Result<BmsModel> {
    let content = std::fs::read(path)?;
    load_bms_from_bytes(&content, path)
}

/// Load a BMS model from raw bytes (avoids redundant file reads).
pub fn load_bms_from_bytes(data: &[u8], path: &Path) -> Result<BmsModel> {
    // BMS files are typically Shift_JIS encoded
    let text = decode_bms_content(data);

    parse_bms_text(&text, path)
}

/// Decode BMS content, handling Shift_JIS and UTF-8.
fn decode_bms_content(data: &[u8]) -> String {
    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(data) {
        return s.to_string();
    }

    // Fall back to Shift_JIS using encoding_rs
    let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(data);
    cow.into_owned()
}

/// Parse BMS text content into a BmsModel.
fn parse_bms_text(text: &str, path: &Path) -> Result<BmsModel> {
    let config = default_config();
    let output =
        parse_bms(text, config).map_err(|e| anyhow::anyhow!("failed to parse BMS: {:?}", e))?;

    convert_bms_to_model(&output.bms, path)
}

/// Convert bms-rs Bms struct into our internal BmsModel.
fn convert_bms_to_model(bms: &Bms, path: &Path) -> Result<BmsModel> {
    let mut model = BmsModel {
        path: path.to_string_lossy().to_string(),
        ..Default::default()
    };

    // -- Extract metadata from music_info --
    if let Some(title) = &bms.music_info.title {
        model.title.clone_from(title);
    }
    if let Some(subtitle) = &bms.music_info.subtitle {
        model.subtitle.clone_from(subtitle);
    }
    if let Some(artist) = &bms.music_info.artist {
        model.artist.clone_from(artist);
    }
    if let Some(sub_artist) = &bms.music_info.sub_artist {
        model.subartist.clone_from(sub_artist);
    }
    if let Some(genre) = &bms.music_info.genre {
        model.genre.clone_from(genre);
    }
    if let Some(preview) = &bms.music_info.preview_music {
        model.preview = preview.to_string_lossy().to_string();
    }

    // -- Extract from metadata --
    if let Some(playlevel) = bms.metadata.play_level {
        model.playlevel = playlevel.to_string();
    }
    model.difficulty = bms.metadata.difficulty.unwrap_or(0) as i32;

    // -- BPM --
    model.bpm = bms
        .bpm
        .bpm
        .as_ref()
        .and_then(|d: &bms_rs::bms::Decimal| d.to_f64())
        .unwrap_or(130.0);

    // -- Total --
    model.total = bms
        .judge
        .total
        .as_ref()
        .and_then(|d: &bms_rs::bms::Decimal| d.to_f64())
        .unwrap_or(0.0);

    // -- Judge rank --
    if let Some(rank) = &bms.judge.rank {
        match rank {
            JudgeLevel::VeryHard => {
                model.judge_rank = 0;
                model.judge_rank_type = JudgeRankType::BmsRank;
            }
            JudgeLevel::Hard => {
                model.judge_rank = 1;
                model.judge_rank_type = JudgeRankType::BmsRank;
            }
            JudgeLevel::Normal => {
                model.judge_rank = 2;
                model.judge_rank_type = JudgeRankType::BmsRank;
            }
            JudgeLevel::Easy => {
                model.judge_rank = 3;
                model.judge_rank_type = JudgeRankType::BmsRank;
            }
            JudgeLevel::OtherInt(v) => {
                model.judge_rank = *v as i32;
                model.judge_rank_type = JudgeRankType::BmsDefExRank;
            }
        }
    }

    // Check for DEFEXRANK (higher priority)
    if !bms.judge.exrank_defs.is_empty()
        && let Some(exrank) = bms.judge.exrank_defs.values().next()
        && let JudgeLevel::OtherInt(v) = exrank.judge_level
    {
        model.judge_rank = v as i32;
        model.judge_rank_type = JudgeRankType::BmsDefExRank;
    }

    // -- LN mode --
    model.ln_mode = match bms.repr.ln_mode {
        bms_rs::command::LnMode::Ln => LongNoteMode::LnType1,
        bms_rs::command::LnMode::Cn => LongNoteMode::ChargeNote,
        bms_rs::command::LnMode::Hcn => LongNoteMode::HellChargeNote,
    };

    // -- Player mode --
    model.mode = detect_play_mode(bms);

    // -- WAV definitions --
    for (id, wav_path) in &bms.wav.wav_files {
        model.wav_defs.push(WavDef {
            id: id.as_u32(),
            path: wav_path.to_string_lossy().to_string(),
        });
    }

    // -- BMP definitions --
    for (id, bmp) in &bms.bmp.bmp_files {
        model.bmp_defs.push(super::bms_model::BmpDef {
            id: id.as_u32(),
            path: bmp.file.to_string_lossy().to_string(),
        });
    }

    // -- Build section lengths (measure lengths) --
    let max_track = estimate_max_track(bms);
    let section_lengths = build_section_lengths(bms, max_track);

    // -- Build timing engine --
    let timing_engine = build_timing_engine(bms, &section_lengths);

    // -- Convert BPM changes --
    for (obj_time, bpm_change) in &bms.bpm.bpm_changes {
        let beat_pos = obj_time_to_beat_position(obj_time, &section_lengths);
        model.bpm_changes.push(BpmChange {
            time_us: timing_engine.beat_to_us(beat_pos),
            bpm: bpm_change.bpm.to_f64().unwrap_or(130.0),
        });
    }

    // -- Convert stop events --
    for (obj_time, stop) in &bms.stop.stops {
        let beat_pos = obj_time_to_beat_position(obj_time, &section_lengths);
        let duration_beats = stop.duration.to_f64().unwrap_or(0.0);
        let bpm_at_stop = timing_engine.bpm_at(beat_pos);
        let duration_us = if bpm_at_stop > 0.0 {
            (duration_beats * 60_000_000.0 / bpm_at_stop) as i64
        } else {
            0
        };
        model.stops.push(StopEvent {
            time_us: timing_engine.beat_to_us(beat_pos),
            duration_us,
        });
    }

    // -- Convert notes --
    for note in bms.notes().all_notes() {
        if let Some(key_layout) = KeyLayoutBeat::from_channel_id(note.channel_id) {
            let (side, kind, key) = key_layout.as_tuple();
            if let Some(lane) = key_to_lane(side, key, model.mode) {
                let beat_pos = obj_time_to_beat_position(&note.offset, &section_lengths);
                let time_us = timing_engine.beat_to_us(beat_pos);

                let note_type = match kind {
                    NoteKind::Visible => NoteType::Normal,
                    NoteKind::Invisible => NoteType::Invisible,
                    NoteKind::Long => match model.ln_mode {
                        LongNoteMode::LnType1 => NoteType::LongNote,
                        LongNoteMode::ChargeNote => NoteType::ChargeNote,
                        LongNoteMode::HellChargeNote => NoteType::HellChargeNote,
                    },
                    NoteKind::Landmine => NoteType::Mine,
                };

                model.notes.push(Note {
                    lane,
                    note_type,
                    time_us,
                    end_time_us: 0, // LN end times resolved in post-processing
                    wav_id: note.wav_id.as_u32(),
                    damage: if kind == NoteKind::Landmine {
                        note.wav_id.as_u32() as f64
                    } else {
                        0.0
                    },
                });
            }
        }
    }

    // -- Resolve LN end times --
    resolve_ln_end_times(&mut model.notes);

    // -- Convert BGM events --
    for note in bms.notes().all_notes() {
        if KeyLayoutBeat::from_channel_id(note.channel_id).is_none() {
            let beat_pos = obj_time_to_beat_position(&note.offset, &section_lengths);
            model.bgm_events.push(BgmEvent {
                time_us: timing_engine.beat_to_us(beat_pos),
                wav_id: note.wav_id.as_u32(),
            });
        }
    }

    // -- Convert BGA events --
    for (obj_time, bga) in &bms.bmp.bga_changes {
        let beat_pos = obj_time_to_beat_position(obj_time, &section_lengths);
        let layer = match bga.layer {
            bms_rs::bms::model::obj::BgaLayer::Base => 0,
            bms_rs::bms::model::obj::BgaLayer::Overlay => 1,
            bms_rs::bms::model::obj::BgaLayer::Poor => 2,
            bms_rs::bms::model::obj::BgaLayer::Overlay2 => 3,
            _ => 0,
        };
        model.bga_events.push(BgaEvent {
            time_us: timing_engine.beat_to_us(beat_pos),
            bmp_id: bga.id.as_u32(),
            layer,
        });
    }

    // Sort notes by time
    model.notes.sort_by_key(|n| n.time_us);
    model.bgm_events.sort_by_key(|e| e.time_us);
    model.bga_events.sort_by_key(|e| e.time_us);
    model.bpm_changes.sort_by_key(|e| e.time_us);

    // Validate the model
    model.validate();

    Ok(model)
}

/// Convert bms-rs ObjTime to beat position using section lengths.
fn obj_time_to_beat_position(
    obj_time: &bms_rs::command::time::ObjTime,
    section_lengths: &[f64],
) -> f64 {
    let track = obj_time.track().0;
    let fraction = if obj_time.denominator_u64() > 0 {
        obj_time.numerator() as f64 / obj_time.denominator_u64() as f64
    } else {
        0.0
    };

    // Sum beats for all measures before this track
    let mut total_beats = 0.0;
    // Track is 1-indexed in bms-rs
    for m in 0..track.saturating_sub(1) {
        let len = section_lengths.get(m as usize).copied().unwrap_or(1.0);
        total_beats += len * 4.0;
    }
    // Add fractional position within current measure
    if track > 0 {
        let current_len = section_lengths
            .get((track - 1) as usize)
            .copied()
            .unwrap_or(1.0);
        total_beats += fraction * current_len * 4.0;
    }
    total_beats
}

/// Estimate the maximum track number from all events.
fn estimate_max_track(bms: &Bms) -> u64 {
    let mut max_track = 0u64;
    if let Some(last) = bms.notes().last_obj_time() {
        max_track = max_track.max(last.track().0);
    }
    if let Some(last) = bms.bpm.last_obj_time() {
        max_track = max_track.max(last.track().0);
    }
    for &track in bms.section_len.section_len_changes.keys() {
        max_track = max_track.max(track.0);
    }
    max_track + 1
}

/// Build section (measure) lengths from BMS section_len data.
fn build_section_lengths(bms: &Bms, max_track: u64) -> Vec<f64> {
    let mut lengths = vec![1.0; max_track as usize];
    for (&track, section) in &bms.section_len.section_len_changes {
        if let Some(slot) = lengths.get_mut(track.0.saturating_sub(1) as usize) {
            *slot = section.length.to_f64().unwrap_or(1.0);
        }
    }
    lengths
}

/// Build a timing engine from BMS BPM changes and stops.
fn build_timing_engine(bms: &Bms, section_lengths: &[f64]) -> TimingEngine {
    let initial_bpm = bms
        .bpm
        .bpm
        .as_ref()
        .and_then(|d: &bms_rs::bms::Decimal| d.to_f64())
        .unwrap_or(130.0);

    let mut bpm_events = Vec::new();
    for (obj_time, bpm_change) in &bms.bpm.bpm_changes {
        let beat_pos = obj_time_to_beat_position(obj_time, section_lengths);
        bpm_events.push(BpmEvent {
            beat_position: beat_pos,
            bpm: bpm_change.bpm.to_f64().unwrap_or(130.0),
        });
    }
    // Also process u8 BPM changes (channel 03)
    for (obj_time, &bpm_u8) in &bms.bpm.bpm_changes_u8 {
        let beat_pos = obj_time_to_beat_position(obj_time, section_lengths);
        bpm_events.push(BpmEvent {
            beat_position: beat_pos,
            bpm: bpm_u8 as f64,
        });
    }
    bpm_events.sort_by(|a, b| a.beat_position.partial_cmp(&b.beat_position).unwrap());

    let mut stop_specs = Vec::new();
    for (obj_time, stop) in &bms.stop.stops {
        let beat_pos = obj_time_to_beat_position(obj_time, section_lengths);
        // Stop duration is in 192nds of a measure (4 beats)
        let duration_beats = stop.duration.to_f64().unwrap_or(0.0) / 192.0 * 4.0;
        stop_specs.push(StopSpec {
            beat_position: beat_pos,
            duration_beats,
        });
    }
    stop_specs.sort_by(|a, b| a.beat_position.partial_cmp(&b.beat_position).unwrap());

    TimingEngine::new(initial_bpm, bpm_events, stop_specs)
}

/// Convert bms-rs Key + PlayerSide to internal lane index.
fn key_to_lane(side: PlayerSide, key: Key, mode: PlayMode) -> Option<usize> {
    let player_offset = match side {
        PlayerSide::Player1 => 0,
        PlayerSide::Player2 => mode.lane_count() / mode.player_count(),
    };

    let lane_in_player = match mode {
        PlayMode::Beat5K | PlayMode::Beat10K => match key {
            Key::Key(1) => Some(0),
            Key::Key(2) => Some(1),
            Key::Key(3) => Some(2),
            Key::Key(4) => Some(3),
            Key::Key(5) => Some(4),
            Key::Scratch(1) => Some(5),
            _ => None,
        },
        PlayMode::Beat7K | PlayMode::Beat14K => match key {
            Key::Key(1) => Some(0),
            Key::Key(2) => Some(1),
            Key::Key(3) => Some(2),
            Key::Key(4) => Some(3),
            Key::Key(5) => Some(4),
            Key::Key(6) => Some(5),
            Key::Key(7) => Some(6),
            Key::Scratch(1) => Some(7),
            _ => None,
        },
        PlayMode::PopN9K => match key {
            Key::Key(n) if (1..=9).contains(&n) => Some((n - 1) as usize),
            _ => None,
        },
        PlayMode::Keyboard24K | PlayMode::Keyboard24KDouble => match key {
            Key::Key(n) if (1..=26).contains(&n) => Some((n - 1) as usize),
            _ => None,
        },
    };

    lane_in_player.map(|l| l + player_offset)
}

/// Resolve LN end times by pairing LN start/end notes.
/// In BMS, LN notes come in pairs on the same lane.
fn resolve_ln_end_times(notes: &mut [Note]) {
    let lane_count = notes.iter().map(|n| n.lane).max().unwrap_or(0) + 1;

    for lane in 0..lane_count {
        let mut ln_start: Option<usize> = None;

        let mut indices: Vec<usize> = notes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.lane == lane && n.is_long_note())
            .map(|(i, _)| i)
            .collect();
        indices.sort_by_key(|&i| notes[i].time_us);

        for idx in indices {
            if let Some(start_idx) = ln_start {
                notes[start_idx].end_time_us = notes[idx].time_us;
                ln_start = None;
                notes[idx].note_type = NoteType::Invisible;
            } else {
                ln_start = Some(idx);
            }
        }
    }
}

/// Detect play mode from BMS data.
fn detect_play_mode(bms: &Bms) -> PlayMode {
    let is_double = matches!(
        bms.metadata.player,
        Some(PlayerMode::Double | PlayerMode::Two)
    );

    let mut has_key6 = false;
    let mut has_key7 = false;
    for note in bms.notes().all_notes() {
        if let Some(key_layout) = KeyLayoutBeat::from_channel_id(note.channel_id) {
            match key_layout.key() {
                Key::Key(6) => has_key6 = true,
                Key::Key(7) => has_key7 = true,
                _ => {}
            }
        }
    }

    if is_double {
        if has_key6 || has_key7 {
            PlayMode::Beat14K
        } else {
            PlayMode::Beat10K
        }
    } else if has_key6 || has_key7 {
        PlayMode::Beat7K
    } else {
        PlayMode::Beat5K
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf8() {
        let data = "hello world".as_bytes();
        assert_eq!(decode_bms_content(data), "hello world");
    }

    #[test]
    fn key_to_lane_beat7k() {
        assert_eq!(
            key_to_lane(PlayerSide::Player1, Key::Key(1), PlayMode::Beat7K),
            Some(0)
        );
        assert_eq!(
            key_to_lane(PlayerSide::Player1, Key::Key(7), PlayMode::Beat7K),
            Some(6)
        );
        assert_eq!(
            key_to_lane(PlayerSide::Player1, Key::Scratch(1), PlayMode::Beat7K),
            Some(7)
        );
    }

    #[test]
    fn key_to_lane_beat14k() {
        assert_eq!(
            key_to_lane(PlayerSide::Player2, Key::Key(1), PlayMode::Beat14K),
            Some(8)
        );
        assert_eq!(
            key_to_lane(PlayerSide::Player2, Key::Scratch(1), PlayMode::Beat14K),
            Some(15)
        );
    }

    #[test]
    fn key_to_lane_popn9k() {
        for i in 1..=9 {
            assert_eq!(
                key_to_lane(PlayerSide::Player1, Key::Key(i), PlayMode::PopN9K),
                Some((i - 1) as usize)
            );
        }
    }

    #[test]
    fn obj_time_to_beat_position_basic() {
        let section_lengths = vec![1.0; 10];
        // Track 1 (first measure), fraction 0 -> beat 0
        let obj_time =
            bms_rs::command::time::ObjTime::new(1, 0, std::num::NonZeroU64::new(1).unwrap());
        assert_eq!(obj_time_to_beat_position(&obj_time, &section_lengths), 0.0);

        // Track 2 (second measure), fraction 0 -> beat 4
        let obj_time =
            bms_rs::command::time::ObjTime::new(2, 0, std::num::NonZeroU64::new(1).unwrap());
        assert_eq!(obj_time_to_beat_position(&obj_time, &section_lengths), 4.0);

        // Track 1, fraction 1/2 -> beat 2
        let obj_time =
            bms_rs::command::time::ObjTime::new(1, 1, std::num::NonZeroU64::new(2).unwrap());
        assert_eq!(obj_time_to_beat_position(&obj_time, &section_lengths), 2.0);
    }

    #[test]
    fn load_sample_bms() {
        let path = Path::new("bms/bms-001/fumble_7n.bms");
        if !path.exists() {
            return;
        }

        let model = load_bms(path).expect("failed to load BMS");
        assert!(!model.title.is_empty(), "title should not be empty");
        assert!(model.bpm > 0.0, "BPM should be positive");
        assert!(model.total_notes() > 0, "should have notes");
        assert_eq!(model.mode, PlayMode::Beat7K, "should be 7K mode");
    }
}

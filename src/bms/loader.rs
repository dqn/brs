use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use bms_rs::bms::prelude::*;
use fraction::Fraction;

use super::{
    BgmEvent, BmsError, BpmChange, Chart, MeasureLength, Metadata, Note, NoteChannel, NoteType,
    StopEvent, TimingData,
};

pub struct BmsLoader;

impl BmsLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<(Chart, HashMap<u32, String>)> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path).map_err(|e| BmsError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;

        let output = parse_bms(&source, default_config())
            .map_err(|e| BmsError::Parse(format!("{:?}", e)))?;

        let bms = output.bms;

        let chart = Self::convert_to_chart(&bms)?;
        let wav_files = Self::extract_wav_files(&bms);

        Ok((chart, wav_files))
    }

    fn convert_to_chart(bms: &Bms) -> Result<Chart> {
        let metadata = Self::extract_metadata(bms);
        let timing_data = Self::extract_timing_data(bms);
        let (notes, bgm_events) = Self::extract_notes(bms, &timing_data)?;

        Ok(Chart {
            metadata,
            timing_data,
            notes,
            bgm_events,
        })
    }

    fn extract_metadata(bms: &Bms) -> Metadata {
        let music_info = &bms.music_info;
        let meta = &bms.metadata;
        let judge = &bms.judge;
        let initial_bpm = bms.bpm.bpm.as_ref().map(decimal_to_f64).unwrap_or(130.0);

        Metadata {
            title: music_info.title.as_deref().unwrap_or("Unknown").to_string(),
            subtitle: music_info.subtitle.clone(),
            artist: music_info
                .artist
                .as_deref()
                .unwrap_or("Unknown")
                .to_string(),
            genre: music_info.genre.as_deref().unwrap_or("").to_string(),
            bpm: initial_bpm,
            play_level: meta.play_level.unwrap_or(0) as u32,
            rank: judge.rank.map(judge_level_to_u32).unwrap_or(2),
            total: judge.total.as_ref().map(decimal_to_f64).unwrap_or(100.0),
        }
    }

    fn extract_timing_data(bms: &Bms) -> TimingData {
        let initial_bpm = bms.bpm.bpm.as_ref().map(decimal_to_f64).unwrap_or(130.0);

        let bpm_changes: Vec<BpmChange> = bms
            .bpm
            .bpm_changes
            .iter()
            .map(|(time, change)| {
                let track = time.track().0;
                BpmChange {
                    measure: track as u32,
                    position: obj_time_to_fraction(time),
                    bpm: decimal_to_f64(&change.bpm),
                }
            })
            .collect();

        let stops: Vec<StopEvent> = bms
            .stop
            .stops
            .iter()
            .map(|(time, stop)| {
                let track = time.track().0;
                StopEvent {
                    measure: track as u32,
                    position: obj_time_to_fraction(time),
                    duration_192: (decimal_to_f64(&stop.duration) * 192.0) as u32,
                }
            })
            .collect();

        let measure_lengths: Vec<MeasureLength> = bms
            .section_len
            .section_len_changes
            .iter()
            .map(|(track, change)| MeasureLength {
                measure: track.0 as u32,
                length: decimal_to_f64(&change.length),
            })
            .collect();

        TimingData {
            initial_bpm,
            bpm_changes,
            stops,
            measure_lengths,
        }
    }

    fn extract_notes(bms: &Bms, timing: &TimingData) -> Result<(Vec<Note>, Vec<BgmEvent>)> {
        let mut notes = Vec::new();
        let mut bgm_events = Vec::new();

        for obj in bms.wav.notes.all_notes() {
            if obj.wav_id.is_null() {
                continue;
            }

            let track = obj.offset.track().0;
            let measure = track as u32;
            let position = obj_time_to_fraction(&obj.offset);
            let keysound_id: u32 = obj.wav_id.into();

            let is_bgm = obj
                .channel_id
                .try_into_map::<KeyLayoutBeat>()
                .is_none_or(|map| !map.kind().is_displayable());

            if is_bgm {
                let time_ms = super::calculate_time_ms(measure, position, timing);
                bgm_events.push(BgmEvent {
                    measure,
                    position,
                    time_ms,
                    keysound_id,
                });
            } else if let Some(note_channel) = channel_id_to_note_channel(&obj.channel_id) {
                let time_ms = super::calculate_time_ms(measure, position, timing);
                let note_type = channel_id_to_note_type(&obj.channel_id);
                notes.push(Note {
                    measure,
                    position,
                    time_ms,
                    channel: note_channel,
                    keysound_id,
                    note_type,
                });
            }
        }

        notes.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));
        bgm_events.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));

        Ok((notes, bgm_events))
    }

    fn extract_wav_files(bms: &Bms) -> HashMap<u32, String> {
        bms.wav
            .wav_files
            .iter()
            .map(|(id, path)| {
                let id_num: u32 = (*id).into();
                (id_num, path.to_string_lossy().to_string())
            })
            .collect()
    }
}

fn decimal_to_f64(d: &Decimal) -> f64 {
    use std::str::FromStr;
    f64::from_str(&d.to_string()).unwrap_or(0.0)
}

fn judge_level_to_u32(level: JudgeLevel) -> u32 {
    match level {
        JudgeLevel::VeryHard => 0,
        JudgeLevel::Hard => 1,
        JudgeLevel::Normal => 2,
        JudgeLevel::Easy => 3,
        JudgeLevel::OtherInt(v) => v as u32,
    }
}

fn obj_time_to_fraction(time: &ObjTime) -> Fraction {
    Fraction::new(time.numerator(), time.denominator().get())
}

fn channel_id_to_note_channel(channel_id: &NoteChannelId) -> Option<NoteChannel> {
    let mapping = channel_id.try_into_map::<KeyLayoutBeat>()?;
    let key = mapping.key();

    match key {
        Key::Scratch(_) => Some(NoteChannel::Scratch),
        Key::Key(1) => Some(NoteChannel::Key1),
        Key::Key(2) => Some(NoteChannel::Key2),
        Key::Key(3) => Some(NoteChannel::Key3),
        Key::Key(4) => Some(NoteChannel::Key4),
        Key::Key(5) => Some(NoteChannel::Key5),
        Key::Key(6) => Some(NoteChannel::Key6),
        Key::Key(7) => Some(NoteChannel::Key7),
        _ => None,
    }
}

fn channel_id_to_note_type(channel_id: &NoteChannelId) -> NoteType {
    if let Some(mapping) = channel_id.try_into_map::<KeyLayoutBeat>() {
        match mapping.kind() {
            NoteKind::Visible => NoteType::Normal,
            NoteKind::Invisible => NoteType::Invisible,
            NoteKind::Long => NoteType::Normal,
            NoteKind::Landmine => NoteType::Landmine,
        }
    } else {
        NoteType::Normal
    }
}

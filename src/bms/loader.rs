use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::Result;
use bms_rs::bms::prelude::*;
use fraction::Fraction;

use super::{
    BgaEvent, BgaLayer, BgmEvent, BmsError, Bmson, BpmChange, Chart, LnType, MeasureLength,
    Metadata, Note, NoteChannel, NoteType, PlayMode, StopEvent, TimingData,
};

pub struct BmsLoader;

/// Result of loading a BMS file
pub struct BmsLoadResult {
    pub chart: Chart,
    pub wav_files: HashMap<u32, String>,
    pub bmp_files: HashMap<u32, String>,
}

impl BmsLoader {
    #[allow(dead_code)] // Used in future song select implementation
    pub fn load<P: AsRef<Path>>(path: P) -> Result<(Chart, HashMap<u32, String>)> {
        let result = Self::load_full(path)?;
        Ok((result.chart, result.wav_files))
    }

    pub fn load_full<P: AsRef<Path>>(path: P) -> Result<BmsLoadResult> {
        let path = path.as_ref();

        // Check for BMSON format
        let is_bmson = path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bmson"));

        if is_bmson {
            return Self::load_bmson(path);
        }

        Self::load_bms_or_pms(path)
    }

    fn load_bmson(path: &Path) -> Result<BmsLoadResult> {
        let content = std::fs::read_to_string(path).map_err(|e| BmsError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;

        let bmson: Bmson =
            serde_json::from_str(&content).map_err(|e| BmsError::Parse(format!("{}", e)))?;

        let chart = bmson.to_chart()?;
        if chart.metadata.play_mode != PlayMode::Bms7Key {
            return Err(BmsError::UnsupportedPlayMode {
                mode: format!("{:?}", chart.metadata.play_mode),
            }
            .into());
        }
        let wav_files = bmson.collect_wav_files();
        let bmp_files = bmson.collect_bmp_files();

        Ok(BmsLoadResult {
            chart,
            wav_files,
            bmp_files,
        })
    }

    fn load_bms_or_pms(path: &Path) -> Result<BmsLoadResult> {
        let source = super::read_bms_file(path)?;

        // Detect PMS by file extension
        let is_pms = path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pms"));

        // Parse BMS/PMS - bms-rs handles both formats
        let output = parse_bms(&source, default_config())
            .map_err(|e| BmsError::Parse(format!("{:?}", e)))?;

        let bms = output.bms;

        // Detect play mode: PMS > DP > SP
        let play_mode = if is_pms {
            PlayMode::Pms9Key
        } else if Self::has_p2_notes(&bms) {
            PlayMode::Dp14Key
        } else {
            PlayMode::Bms7Key
        };

        if play_mode != PlayMode::Bms7Key {
            return Err(BmsError::UnsupportedPlayMode {
                mode: format!("{:?}", play_mode),
            }
            .into());
        }

        let mut chart = Self::convert_to_chart(&bms, play_mode)?;
        let wav_files = Self::extract_wav_files(&bms);
        let bmp_files = Self::extract_bmp_files(&bms);

        // Extract BGA events directly from source to work around bms-rs limitation
        // (bms-rs overwrites events at the same time with different layers)
        chart.bga_events = Self::extract_bga_events_from_source(&source, &chart.timing_data);

        Ok(BmsLoadResult {
            chart,
            wav_files,
            bmp_files,
        })
    }

    /// Check if the BMS has P2 notes (channels 21-29)
    fn has_p2_notes(bms: &Bms) -> bool {
        bms.wav.notes.all_notes().any(|obj| {
            if let Some(mapping) = obj.channel_id.try_into_map::<KeyLayoutBeat>() {
                match mapping.key() {
                    Key::Scratch(2) => true,       // P2 scratch
                    Key::Key(n) if n >= 8 => true, // P2 keys are 8-14
                    _ => false,
                }
            } else {
                false
            }
        })
    }

    fn convert_to_chart(bms: &Bms, play_mode: PlayMode) -> Result<Chart> {
        let metadata = Self::extract_metadata(bms, play_mode);
        let timing_data = Self::extract_timing_data(bms);
        let (notes, bgm_events) = Self::extract_notes(bms, &timing_data, play_mode)?;
        let bga_events = Self::extract_bga_events(bms, &timing_data);

        Ok(Chart {
            metadata,
            timing_data,
            notes,
            bgm_events,
            bga_events,
        })
    }

    fn extract_metadata(bms: &Bms, play_mode: PlayMode) -> Metadata {
        let music_info = &bms.music_info;
        let meta = &bms.metadata;
        let judge = &bms.judge;
        let initial_bpm = bms.bpm.bpm.as_ref().map(decimal_to_f64).unwrap_or(130.0);

        let ln_type = match bms.repr.ln_mode {
            LnMode::Ln => LnType::Ln,
            LnMode::Cn => LnType::Cn,
            LnMode::Hcn => LnType::Hcn,
        };

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
            ln_type,
            play_mode,
        }
    }

    fn extract_timing_data(bms: &Bms) -> TimingData {
        let initial_bpm = bms.bpm.bpm.as_ref().map(decimal_to_f64).unwrap_or(130.0);

        let mut bpm_changes: Vec<BpmChange> = Vec::new();

        for (time, change) in &bms.bpm.bpm_changes {
            let track = time.track().0;
            bpm_changes.push(BpmChange {
                measure: track as u32,
                position: obj_time_to_fraction(time),
                bpm: decimal_to_f64(&change.bpm),
            });
        }

        for (time, bpm) in &bms.bpm.bpm_changes_u8 {
            if *bpm == 0 {
                continue;
            }
            let track = time.track().0;
            bpm_changes.push(BpmChange {
                measure: track as u32,
                position: obj_time_to_fraction(time),
                bpm: *bpm as f64,
            });
        }

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

    fn extract_notes(
        bms: &Bms,
        timing: &TimingData,
        play_mode: PlayMode,
    ) -> Result<(Vec<Note>, Vec<BgmEvent>)> {
        use super::MAX_LANE_COUNT;

        let mut notes: Vec<Note> = Vec::new();
        let mut bgm_events = Vec::new();
        // Track pending long note starts per lane.
        // MAX_LANE_COUNT (16) accommodates:
        // - BMS 7-key SP: 8 lanes (scratch + 7 keys)
        // - BMS 14-key DP: 16 lanes (2 scratches + 14 keys)
        // - PMS 9-key: 9 lanes
        let mut pending_ln_starts: [Option<(usize, f64)>; MAX_LANE_COUNT] = [None; MAX_LANE_COUNT];

        for obj in bms.wav.notes.all_notes() {
            if obj.wav_id.is_null() {
                continue;
            }

            let track = obj.offset.track().0;
            let measure = track as u32;
            let position = obj_time_to_fraction(&obj.offset);
            let keysound_id: u32 = obj.wav_id.into();

            let is_bgm = !is_channel_displayable(&obj.channel_id, play_mode);

            if is_bgm {
                let time_ms = super::calculate_time_ms(measure, position, timing);
                bgm_events.push(BgmEvent {
                    measure,
                    position,
                    time_ms,
                    keysound_id,
                });
            } else if let Some(note_channel) =
                channel_id_to_note_channel(&obj.channel_id, play_mode)
            {
                let time_ms = super::calculate_time_ms(measure, position, timing);
                let note_kind = get_note_kind(&obj.channel_id, play_mode);

                let lane_idx = note_channel.lane_index_for_mode(play_mode);

                if note_kind == Some(NoteKind::Long) {
                    // Long note handling: pair LongStart and LongEnd
                    if let Some((start_idx, _start_time)) = pending_ln_starts[lane_idx].take() {
                        // This is the end of a long note
                        notes[start_idx].long_end_time_ms = Some(time_ms);
                        notes.push(Note {
                            measure,
                            position,
                            time_ms,
                            channel: note_channel,
                            keysound_id,
                            note_type: NoteType::LongEnd,
                            long_end_time_ms: None,
                        });
                    } else {
                        // This is the start of a long note
                        let idx = notes.len();
                        pending_ln_starts[lane_idx] = Some((idx, time_ms));
                        notes.push(Note {
                            measure,
                            position,
                            time_ms,
                            channel: note_channel,
                            keysound_id,
                            note_type: NoteType::LongStart,
                            long_end_time_ms: None,
                        });
                    }
                } else {
                    let note_type = channel_id_to_note_type(&obj.channel_id, play_mode);
                    notes.push(Note {
                        measure,
                        position,
                        time_ms,
                        channel: note_channel,
                        keysound_id,
                        note_type,
                        long_end_time_ms: None,
                    });
                }
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

    fn extract_bmp_files(bms: &Bms) -> HashMap<u32, String> {
        bms.bmp
            .bmp_files
            .iter()
            .map(|(id, bmp)| {
                let id_num: u32 = (*id).into();
                (id_num, bmp.file.to_string_lossy().to_string())
            })
            .collect()
    }

    fn extract_bga_events(bms: &Bms, timing: &TimingData) -> Vec<BgaEvent> {
        let mut events = Vec::new();

        // All BGA events from bga_changes (unified in bms-rs 0.10+)
        for (time, bga) in &bms.bmp.bga_changes {
            let track = time.track().0 as u32;
            let position = obj_time_to_fraction(time);
            let time_ms = super::calculate_time_ms(track, position, timing);
            let bga_id: u32 = bga.id.into();

            // Map bms-rs BgaLayer to our internal BgaLayer
            let layer = match bga.layer {
                bms_rs::bms::prelude::BgaLayer::Base => BgaLayer::Base,
                bms_rs::bms::prelude::BgaLayer::Poor => BgaLayer::Poor,
                bms_rs::bms::prelude::BgaLayer::Overlay
                | bms_rs::bms::prelude::BgaLayer::Overlay2 => BgaLayer::Overlay,
                _ => BgaLayer::Overlay, // Treat unknown layers as Overlay for compatibility
            };

            events.push(BgaEvent {
                time_ms,
                bga_id,
                layer,
            });
        }

        events.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));
        events
    }

    /// Extract BGA events directly from BMS source text.
    ///
    /// This works around bms-rs limitation where events at the same time overwrite each other.
    ///
    /// # Limitations
    /// - Lines longer than 10,000 characters are skipped as a DoS protection measure.
    /// - Only channels 04, 06, 07, 0A are processed.
    /// - Object data is limited to 10,000 characters to prevent excessive memory allocation.
    fn extract_bga_events_from_source(source: &str, timing: &TimingData) -> Vec<BgaEvent> {
        use regex::Regex;
        use std::sync::OnceLock;

        // DoS protection: skip excessively long lines
        const MAX_LINE_LENGTH: usize = 10_000;
        // Maximum object data length to process
        const MAX_DATA_LENGTH: usize = 10_000;

        let mut events = Vec::new();

        // Pattern: #xxxYY:data where xxx=measure, YY=channel, data=object IDs
        // Channel 04 = BGA Base, 06 = BGA Poor, 07 = BGA Layer, 0A = BGA Layer2
        // Use OnceLock to compile regex only once for better performance
        static BGA_REGEX: OnceLock<Regex> = OnceLock::new();
        let re = BGA_REGEX.get_or_init(|| {
            Regex::new(r"(?i)^#(\d{3})(04|06|07|0A):([0-9A-Za-z]+)")
                .expect("BGA regex pattern is invalid")
        });

        for line in source.lines() {
            // Skip lines that are too long (DoS protection)
            if line.len() > MAX_LINE_LENGTH {
                continue;
            }

            let trimmed = line.trim();
            if let Some(caps) = re.captures(trimmed) {
                let measure: u32 = caps[1].parse().unwrap_or(0);
                let channel = &caps[2].to_uppercase();
                let data = &caps[3];

                // Skip excessively long data
                if data.len() > MAX_DATA_LENGTH {
                    continue;
                }

                let layer = match channel.as_str() {
                    "04" => BgaLayer::Base,
                    "06" => BgaLayer::Poor,
                    "07" => BgaLayer::Overlay,
                    "0A" => BgaLayer::Overlay,
                    _ => continue,
                };

                // Parse object IDs (2 characters each, base36)
                let obj_count = data.len() / 2;
                for i in 0..obj_count {
                    let obj_str = &data[i * 2..(i + 1) * 2];
                    if obj_str == "00" {
                        continue; // Skip empty slots
                    }

                    // Parse base36 object ID
                    let bga_id = u32::from_str_radix(obj_str, 36).unwrap_or(0);
                    if bga_id == 0 {
                        continue;
                    }

                    // Calculate position within measure
                    let position = Fraction::new(i as u64, obj_count as u64);
                    let time_ms = super::calculate_time_ms(measure, position, timing);

                    events.push(BgaEvent {
                        time_ms,
                        bga_id,
                        layer,
                    });
                }
            }
        }

        events.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));
        events
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

/// Map channel ID to NoteChannel for BMS 7-key SP mode
fn channel_id_to_note_channel_bms(channel_id: &NoteChannelId) -> Option<NoteChannel> {
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

/// Map channel ID to NoteChannel for BMS 14-key DP mode
/// P1: channels 11-19 (scratch=16, keys=11-15,18-19)
/// P2: channels 21-29 (scratch=26, keys=21-25,28-29)
fn channel_id_to_note_channel_dp(channel_id: &NoteChannelId) -> Option<NoteChannel> {
    let mapping = channel_id.try_into_map::<KeyLayoutBeat>()?;
    let key = mapping.key();

    match key {
        // P1 scratch and keys
        Key::Scratch(1) => Some(NoteChannel::Scratch), // P1 scratch
        Key::Key(1) => Some(NoteChannel::Key1),
        Key::Key(2) => Some(NoteChannel::Key2),
        Key::Key(3) => Some(NoteChannel::Key3),
        Key::Key(4) => Some(NoteChannel::Key4),
        Key::Key(5) => Some(NoteChannel::Key5),
        Key::Key(6) => Some(NoteChannel::Key6),
        Key::Key(7) => Some(NoteChannel::Key7),
        // P2 scratch and keys (map to Key8-14 and Scratch2)
        Key::Scratch(2) => Some(NoteChannel::Scratch2), // P2 scratch
        Key::Key(8) => Some(NoteChannel::Key8),         // P2 Key1
        Key::Key(9) => Some(NoteChannel::Key9),         // P2 Key2
        Key::Key(10) => Some(NoteChannel::Key10),       // P2 Key3
        Key::Key(11) => Some(NoteChannel::Key11),       // P2 Key4
        Key::Key(12) => Some(NoteChannel::Key12),       // P2 Key5
        Key::Key(13) => Some(NoteChannel::Key13),       // P2 Key6
        Key::Key(14) => Some(NoteChannel::Key14),       // P2 Key7
        _ => None,
    }
}

/// Map channel ID to NoteChannel for PMS 9-key mode
/// PMS uses KeyLayoutPms which maps:
/// - P1 Key1-5 → Key1-5 (channels 11-15)
/// - P2 Key2-5 → Key6-9 (channels 22-25)
fn channel_id_to_note_channel_pms(channel_id: &NoteChannelId) -> Option<NoteChannel> {
    let mapping = channel_id.try_into_map::<KeyLayoutPms>()?;
    let key = mapping.key();

    match key {
        Key::Key(1) => Some(NoteChannel::Key1),
        Key::Key(2) => Some(NoteChannel::Key2),
        Key::Key(3) => Some(NoteChannel::Key3),
        Key::Key(4) => Some(NoteChannel::Key4),
        Key::Key(5) => Some(NoteChannel::Key5),
        Key::Key(6) => Some(NoteChannel::Key6),
        Key::Key(7) => Some(NoteChannel::Key7),
        Key::Key(8) => Some(NoteChannel::Key8),
        Key::Key(9) => Some(NoteChannel::Key9),
        _ => None, // No scratch in PMS
    }
}

/// Map channel ID to NoteChannel based on play mode
fn channel_id_to_note_channel(
    channel_id: &NoteChannelId,
    play_mode: PlayMode,
) -> Option<NoteChannel> {
    match play_mode {
        PlayMode::Bms7Key => channel_id_to_note_channel_bms(channel_id),
        PlayMode::Pms9Key => channel_id_to_note_channel_pms(channel_id),
        PlayMode::Dp14Key => channel_id_to_note_channel_dp(channel_id),
    }
}

/// Check if channel is displayable based on play mode
fn is_channel_displayable(channel_id: &NoteChannelId, play_mode: PlayMode) -> bool {
    match play_mode {
        PlayMode::Bms7Key | PlayMode::Dp14Key => channel_id
            .try_into_map::<KeyLayoutBeat>()
            .is_some_and(|map| map.kind().is_displayable()),
        PlayMode::Pms9Key => channel_id
            .try_into_map::<KeyLayoutPms>()
            .is_some_and(|map| map.kind().is_displayable()),
    }
}

/// Get note kind from channel ID based on play mode
fn get_note_kind(channel_id: &NoteChannelId, play_mode: PlayMode) -> Option<NoteKind> {
    match play_mode {
        PlayMode::Bms7Key | PlayMode::Dp14Key => {
            channel_id.try_into_map::<KeyLayoutBeat>().map(|m| m.kind())
        }
        PlayMode::Pms9Key => channel_id.try_into_map::<KeyLayoutPms>().map(|m| m.kind()),
    }
}

fn channel_id_to_note_type(channel_id: &NoteChannelId, play_mode: PlayMode) -> NoteType {
    if let Some(kind) = get_note_kind(channel_id, play_mode) {
        match kind {
            NoteKind::Visible => NoteType::Normal,
            NoteKind::Invisible => NoteType::Invisible,
            NoteKind::Long => NoteType::Normal,
            NoteKind::Landmine => NoteType::Landmine,
        }
    } else {
        NoteType::Normal
    }
}

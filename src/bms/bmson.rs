use std::collections::HashMap;

use anyhow::{Result, anyhow};
use fraction::Fraction;
use serde::Deserialize;

use super::{
    BgaEvent, BgaLayer, BgmEvent, BpmChange, Chart, LnType, Metadata, Note, NoteChannel, NoteType,
    PlayMode, StopEvent, TimingData,
};

/// Ticks per beat in BMSON format (standard resolution)
const TICKS_PER_BEAT: i64 = 240;

/// Ticks per measure (4/4 time signature)
const TICKS_PER_MEASURE: i64 = TICKS_PER_BEAT * 4;

/// Root BMSON structure
#[derive(Debug, Deserialize)]
pub struct Bmson {
    #[allow(dead_code)]
    pub version: String,
    pub info: BmsonInfo,
    #[serde(default)]
    #[allow(dead_code)]
    pub lines: Vec<BarLine>,
    #[serde(default)]
    pub bpm_events: Vec<BmsonBpmEvent>,
    #[serde(default)]
    pub stop_events: Vec<BmsonStopEvent>,
    pub sound_channels: Vec<SoundChannel>,
    #[serde(default)]
    pub bga: Option<BgaData>,
}

/// Metadata information
#[derive(Debug, Deserialize)]
pub struct BmsonInfo {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub subartists: Vec<String>,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub mode_hint: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub chart_name: String,
    #[serde(default)]
    pub level: u32,
    pub init_bpm: f64,
    #[serde(default)]
    pub total: Option<f64>,
    #[serde(default)]
    pub judge_rank: Option<u32>,
}

/// Bar line marker
#[derive(Debug, Deserialize)]
pub struct BarLine {
    #[allow(dead_code)]
    pub y: i64,
}

/// BPM change event
#[derive(Debug, Deserialize)]
pub struct BmsonBpmEvent {
    pub y: i64,
    pub bpm: f64,
}

/// Stop event
#[derive(Debug, Deserialize)]
pub struct BmsonStopEvent {
    pub y: i64,
    pub duration: i64,
}

/// Sound channel with notes
#[derive(Debug, Deserialize)]
pub struct SoundChannel {
    pub name: String,
    pub notes: Vec<BmsonNote>,
}

/// Individual note
#[derive(Debug, Deserialize)]
pub struct BmsonNote {
    /// Lane: 0=BGM, 1-7=keys, 8=scratch (for beat-7k)
    /// For popn-9k: 0=BGM, 1-9=keys
    pub x: i32,
    /// Position in ticks (resolution: 240 per beat)
    pub y: i64,
    /// Long note length in ticks (0 for normal notes)
    #[serde(default)]
    pub l: i64,
    /// Continue flag (for sliced sounds)
    #[serde(default)]
    #[allow(dead_code)]
    pub c: bool,
}

/// BGA (Background Animation) data
#[derive(Debug, Deserialize)]
pub struct BgaData {
    #[serde(default)]
    pub bga_header: Vec<BgaHeader>,
    #[serde(default)]
    pub bga_events: Vec<BmsonBgaEvent>,
    #[serde(default)]
    pub layer_events: Vec<BmsonBgaEvent>,
    #[serde(default)]
    pub poor_events: Vec<BmsonBgaEvent>,
}

/// BGA file header
#[derive(Debug, Deserialize)]
pub struct BgaHeader {
    pub id: u32,
    pub name: String,
}

/// BGA event
#[derive(Debug, Deserialize)]
pub struct BmsonBgaEvent {
    pub y: i64,
    pub id: u32,
}

/// Result of parsing BMSON
#[allow(dead_code)]
pub struct BmsonLoadResult {
    pub chart: Chart,
    pub wav_files: HashMap<u32, String>,
    pub bmp_files: HashMap<u32, String>,
}

impl Bmson {
    /// Convert BMSON to Chart structure
    pub fn to_chart(&self) -> Result<Chart> {
        let play_mode = self.detect_play_mode();
        let timing_data = self.build_timing_data();
        let metadata = self.build_metadata(play_mode);
        let (notes, bgm_events) = self.convert_notes(&timing_data, play_mode)?;
        let bga_events = self.convert_bga_events(&timing_data);

        Ok(Chart {
            metadata,
            timing_data,
            notes,
            bgm_events,
            bga_events,
        })
    }

    /// Collect WAV files mapping
    pub fn collect_wav_files(&self) -> HashMap<u32, String> {
        self.sound_channels
            .iter()
            .enumerate()
            .map(|(i, channel)| (i as u32, channel.name.clone()))
            .collect()
    }

    /// Collect BMP files mapping
    pub fn collect_bmp_files(&self) -> HashMap<u32, String> {
        self.bga
            .as_ref()
            .map(|bga| {
                bga.bga_header
                    .iter()
                    .map(|header| (header.id, header.name.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Detect play mode from mode_hint
    fn detect_play_mode(&self) -> PlayMode {
        match self.info.mode_hint.as_str() {
            "popn-5k" | "popn-9k" => PlayMode::Pms9Key,
            _ => PlayMode::Bms7Key, // "beat-5k", "beat-7k", "beat-10k", "beat-14k"
        }
    }

    /// Build timing data from BMSON events
    fn build_timing_data(&self) -> TimingData {
        let bpm_changes = self
            .bpm_events
            .iter()
            .map(|event| {
                let (measure, position) = ticks_to_measure_position(event.y);
                BpmChange {
                    measure,
                    position,
                    bpm: event.bpm,
                }
            })
            .collect();

        // BMSON stop duration is in ticks, convert to 192 units
        let stops = self
            .stop_events
            .iter()
            .map(|event| {
                let (measure, position) = ticks_to_measure_position(event.y);
                // Convert ticks to 192 units: ticks / 240 * 48 = ticks / 5
                let duration_192 = (event.duration as f64 / TICKS_PER_BEAT as f64 * 48.0) as u32;
                StopEvent {
                    measure,
                    position,
                    duration_192,
                }
            })
            .collect();

        TimingData {
            initial_bpm: self.info.init_bpm,
            bpm_changes,
            stops,
            measure_lengths: Vec::new(), // BMSON uses lines for bar lines, not measure lengths
        }
    }

    /// Build metadata
    fn build_metadata(&self, play_mode: PlayMode) -> Metadata {
        // Convert judge_rank (BMSON uses percentage, default 100)
        // BMS rank: 0=VERY HARD, 1=HARD, 2=NORMAL, 3=EASY
        let rank = match self.info.judge_rank.unwrap_or(100) {
            0..=50 => 0,   // VERY HARD
            51..=75 => 1,  // HARD
            76..=100 => 2, // NORMAL
            _ => 3,        // EASY
        };

        Metadata {
            title: self.info.title.clone(),
            subtitle: if self.info.subtitle.is_empty() {
                None
            } else {
                Some(self.info.subtitle.clone())
            },
            artist: self.info.artist.clone(),
            genre: self.info.genre.clone(),
            bpm: self.info.init_bpm,
            play_level: self.info.level,
            rank,
            total: self.info.total.unwrap_or(300.0),
            ln_type: LnType::Cn, // BMSON defaults to CN-style long notes
            play_mode,
        }
    }

    /// Convert sound channels to notes and BGM events
    fn convert_notes(
        &self,
        _timing: &TimingData,
        play_mode: PlayMode,
    ) -> Result<(Vec<Note>, Vec<BgmEvent>)> {
        let mut notes = Vec::new();
        let mut bgm_events = Vec::new();

        for (keysound_id, channel) in self.sound_channels.iter().enumerate() {
            let keysound_id = keysound_id as u32;

            for bmson_note in &channel.notes {
                let (measure, position) = ticks_to_measure_position(bmson_note.y);
                let time_ms = ticks_to_ms(bmson_note.y, self.info.init_bpm, &self.bpm_events);

                if bmson_note.x == 0 {
                    // BGM channel
                    bgm_events.push(BgmEvent {
                        measure,
                        position,
                        time_ms,
                        keysound_id,
                    });
                } else {
                    // Note channel
                    let note_channel = x_to_channel(bmson_note.x, play_mode)?;

                    if bmson_note.l > 0 {
                        // Long note
                        let end_tick = bmson_note.y + bmson_note.l;
                        let end_time_ms =
                            ticks_to_ms(end_tick, self.info.init_bpm, &self.bpm_events);
                        let (end_measure, end_position) = ticks_to_measure_position(end_tick);

                        // LongStart
                        notes.push(Note {
                            measure,
                            position,
                            time_ms,
                            channel: note_channel,
                            keysound_id,
                            note_type: NoteType::LongStart,
                            long_end_time_ms: Some(end_time_ms),
                        });

                        // LongEnd
                        notes.push(Note {
                            measure: end_measure,
                            position: end_position,
                            time_ms: end_time_ms,
                            channel: note_channel,
                            keysound_id,
                            note_type: NoteType::LongEnd,
                            long_end_time_ms: None,
                        });
                    } else {
                        // Normal note
                        notes.push(Note {
                            measure,
                            position,
                            time_ms,
                            channel: note_channel,
                            keysound_id,
                            note_type: NoteType::Normal,
                            long_end_time_ms: None,
                        });
                    }
                }
            }
        }

        // Sort by time
        notes.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));
        bgm_events.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));

        Ok((notes, bgm_events))
    }

    /// Convert BGA events
    fn convert_bga_events(&self, _timing: &TimingData) -> Vec<BgaEvent> {
        let Some(bga) = &self.bga else {
            return Vec::new();
        };

        let mut events = Vec::new();

        // Base BGA events
        for event in &bga.bga_events {
            let time_ms = ticks_to_ms(event.y, self.info.init_bpm, &self.bpm_events);
            events.push(BgaEvent {
                time_ms,
                bga_id: event.id,
                layer: BgaLayer::Base,
            });
        }

        // Layer events (overlay)
        for event in &bga.layer_events {
            let time_ms = ticks_to_ms(event.y, self.info.init_bpm, &self.bpm_events);
            events.push(BgaEvent {
                time_ms,
                bga_id: event.id,
                layer: BgaLayer::Overlay,
            });
        }

        // Poor events
        for event in &bga.poor_events {
            let time_ms = ticks_to_ms(event.y, self.info.init_bpm, &self.bpm_events);
            events.push(BgaEvent {
                time_ms,
                bga_id: event.id,
                layer: BgaLayer::Poor,
            });
        }

        events.sort_by(|a, b| a.time_ms.total_cmp(&b.time_ms));
        events
    }
}

/// Convert ticks to measure and position
fn ticks_to_measure_position(ticks: i64) -> (u32, Fraction) {
    let measure = (ticks / TICKS_PER_MEASURE) as u32;
    let remainder = ticks % TICKS_PER_MEASURE;
    let position = Fraction::new(remainder as u64, TICKS_PER_MEASURE as u64);
    (measure, position)
}

/// Convert ticks to milliseconds considering BPM changes
fn ticks_to_ms(target_ticks: i64, init_bpm: f64, bpm_events: &[BmsonBpmEvent]) -> f64 {
    let mut time_ms = 0.0;
    let mut current_tick = 0i64;
    let mut current_bpm = init_bpm;

    let ms_per_tick = |bpm: f64| 60_000.0 / bpm / TICKS_PER_BEAT as f64;

    // Sort BPM events by tick position
    let mut sorted_events: Vec<_> = bpm_events.iter().collect();
    sorted_events.sort_by_key(|e| e.y);

    for event in sorted_events {
        if event.y >= target_ticks {
            break;
        }

        // Add time from current_tick to event tick at current BPM
        time_ms += (event.y - current_tick) as f64 * ms_per_tick(current_bpm);
        current_tick = event.y;
        current_bpm = event.bpm;
    }

    // Add remaining time from current_tick to target_ticks
    time_ms += (target_ticks - current_tick) as f64 * ms_per_tick(current_bpm);

    time_ms
}

/// Convert BMSON x lane to NoteChannel
fn x_to_channel(x: i32, play_mode: PlayMode) -> Result<NoteChannel> {
    match play_mode {
        PlayMode::Bms7Key => match x {
            1 => Ok(NoteChannel::Key1),
            2 => Ok(NoteChannel::Key2),
            3 => Ok(NoteChannel::Key3),
            4 => Ok(NoteChannel::Key4),
            5 => Ok(NoteChannel::Key5),
            6 => Ok(NoteChannel::Key6),
            7 => Ok(NoteChannel::Key7),
            8 => Ok(NoteChannel::Scratch),
            _ => Err(anyhow!("Invalid BMSON lane for beat-7k: {}", x)),
        },
        PlayMode::Pms9Key => match x {
            1 => Ok(NoteChannel::Key1),
            2 => Ok(NoteChannel::Key2),
            3 => Ok(NoteChannel::Key3),
            4 => Ok(NoteChannel::Key4),
            5 => Ok(NoteChannel::Key5),
            6 => Ok(NoteChannel::Key6),
            7 => Ok(NoteChannel::Key7),
            8 => Ok(NoteChannel::Key8),
            9 => Ok(NoteChannel::Key9),
            _ => Err(anyhow!("Invalid BMSON lane for popn-9k: {}", x)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticks_to_measure_position() {
        // First beat of first measure
        let (measure, position) = ticks_to_measure_position(0);
        assert_eq!(measure, 0);
        assert_eq!(position, Fraction::new(0u64, TICKS_PER_MEASURE as u64));

        // Second beat of first measure (240 ticks)
        let (measure, position) = ticks_to_measure_position(240);
        assert_eq!(measure, 0);
        assert_eq!(position, Fraction::new(240u64, TICKS_PER_MEASURE as u64));

        // First beat of second measure (960 ticks)
        let (measure, position) = ticks_to_measure_position(960);
        assert_eq!(measure, 1);
        assert_eq!(position, Fraction::new(0u64, TICKS_PER_MEASURE as u64));

        // Third beat of third measure
        let (measure, position) = ticks_to_measure_position(960 * 2 + 480);
        assert_eq!(measure, 2);
        assert_eq!(position, Fraction::new(480u64, TICKS_PER_MEASURE as u64));
    }

    #[test]
    fn test_ticks_to_ms_constant_bpm() {
        let bpm_events = vec![];

        // At 120 BPM: 1 beat = 500ms, 240 ticks = 500ms
        let ms = ticks_to_ms(240, 120.0, &bpm_events);
        assert!((ms - 500.0).abs() < 0.001);

        // At 120 BPM: 4 beats = 2000ms, 960 ticks = 2000ms
        let ms = ticks_to_ms(960, 120.0, &bpm_events);
        assert!((ms - 2000.0).abs() < 0.001);
    }

    #[test]
    fn test_ticks_to_ms_with_bpm_change() {
        // BPM changes from 120 to 240 at tick 480 (half of measure 0)
        let bpm_events = vec![BmsonBpmEvent { y: 480, bpm: 240.0 }];

        // At tick 480: 480/240 * 60000/120 = 1000ms
        let ms = ticks_to_ms(480, 120.0, &bpm_events);
        assert!((ms - 1000.0).abs() < 0.001);

        // At tick 720: 1000ms + (720-480)/240 * 60000/240 = 1000 + 250 = 1250ms
        let ms = ticks_to_ms(720, 120.0, &bpm_events);
        assert!((ms - 1250.0).abs() < 0.001);
    }

    #[test]
    fn test_x_to_channel_bms() {
        assert_eq!(
            x_to_channel(1, PlayMode::Bms7Key).unwrap(),
            NoteChannel::Key1
        );
        assert_eq!(
            x_to_channel(7, PlayMode::Bms7Key).unwrap(),
            NoteChannel::Key7
        );
        assert_eq!(
            x_to_channel(8, PlayMode::Bms7Key).unwrap(),
            NoteChannel::Scratch
        );
        assert!(x_to_channel(9, PlayMode::Bms7Key).is_err());
    }

    #[test]
    fn test_x_to_channel_pms() {
        assert_eq!(
            x_to_channel(1, PlayMode::Pms9Key).unwrap(),
            NoteChannel::Key1
        );
        assert_eq!(
            x_to_channel(9, PlayMode::Pms9Key).unwrap(),
            NoteChannel::Key9
        );
        assert!(x_to_channel(10, PlayMode::Pms9Key).is_err());
    }

    #[test]
    fn test_parse_minimal_bmson() {
        let json = r#"{
            "version": "1.0.0",
            "info": {
                "title": "Test Song",
                "init_bpm": 150.0
            },
            "sound_channels": []
        }"#;

        let bmson: Bmson = serde_json::from_str(json).unwrap();
        assert_eq!(bmson.info.title, "Test Song");
        assert_eq!(bmson.info.init_bpm, 150.0);
        assert!(bmson.sound_channels.is_empty());
    }

    #[test]
    fn test_parse_bmson_with_notes() {
        let json = r#"{
            "version": "1.0.0",
            "info": {
                "title": "Test",
                "init_bpm": 120.0,
                "mode_hint": "beat-7k"
            },
            "sound_channels": [
                {
                    "name": "sound.wav",
                    "notes": [
                        { "x": 0, "y": 0 },
                        { "x": 1, "y": 240 },
                        { "x": 8, "y": 480, "l": 240 }
                    ]
                }
            ]
        }"#;

        let bmson: Bmson = serde_json::from_str(json).unwrap();
        let chart = bmson.to_chart().unwrap();

        // Should have 1 BGM event (x=0)
        assert_eq!(chart.bgm_events.len(), 1);

        // Should have 3 notes: 1 normal + 2 for long note (start + end)
        assert_eq!(chart.notes.len(), 3);

        // First note should be Key1
        assert_eq!(chart.notes[0].channel, NoteChannel::Key1);
        assert_eq!(chart.notes[0].note_type, NoteType::Normal);

        // Long note start (Scratch)
        let ln_start = chart
            .notes
            .iter()
            .find(|n| n.note_type == NoteType::LongStart)
            .unwrap();
        assert_eq!(ln_start.channel, NoteChannel::Scratch);
        assert!(ln_start.long_end_time_ms.is_some());
    }
}

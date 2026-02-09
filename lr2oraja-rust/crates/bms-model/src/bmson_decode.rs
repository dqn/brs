// bmson format decoder
// Reference: Java BMSONDecoder.java (line 44-469)

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::bmson::Bmson;
use crate::mode::PlayMode;
use crate::model::BmsModel;
use crate::note::{BgNote, LnType, Note, NoteType};
use crate::timeline::{BpmChange, StopEvent, TimeLine};

/// bmson file decoder
pub struct BmsonDecoder;

/// Cached timeline entry for y → (time_f64, bpm, stop_us)
struct TimeLineEntry {
    time: f64,
    bpm: f64,
    stop_us: i64,
}

/// Tracks LN ranges for inside-LN detection
#[derive(Clone)]
struct LnRange {
    start_section: f64,
    end_section: f64,
}

impl BmsonDecoder {
    pub fn decode(path: &Path) -> Result<BmsModel> {
        let raw_bytes = std::fs::read(path)?;

        // Compute SHA-256 from raw bytes (MD5 is empty for bmson, matching Java)
        let sha256 = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&raw_bytes);
            format!("{:x}", hasher.finalize())
        };

        let bmson: Bmson = serde_json::from_slice(&raw_bytes)?;

        let mut model = BmsModel {
            sha256,
            md5: String::new(),
            ..Default::default()
        };

        Self::decode_bmson(bmson, &mut model)?;

        Ok(model)
    }

    fn decode_bmson(bmson: Bmson, model: &mut BmsModel) -> Result<()> {
        let info = &bmson.info;

        // Metadata (Java line 62-73)
        model.title = info.title.clone();

        // subtitle = subtitle + " " + "[" + chart_name + "]" (conditional)
        let mut subtitle = info.subtitle.clone();
        if !subtitle.is_empty() && !info.chart_name.is_empty() {
            subtitle.push(' ');
        }
        if !info.chart_name.is_empty() {
            subtitle.push('[');
            subtitle.push_str(&info.chart_name);
            subtitle.push(']');
        }
        model.subtitle = subtitle;

        model.artist = info.artist.clone();
        model.sub_artist = info.subartists.join(",");
        model.genre = info.genre.clone();

        // judge_rank (Java line 76-85)
        // bmson uses DEFEXRANK semantics; raw value equals scaled value
        if info.judge_rank >= 0 {
            model.judge_rank = info.judge_rank;
            model.judge_rank_raw = info.judge_rank;
        }

        // total (Java line 87-92)
        if info.total > 0.0 {
            model.total = info.total;
        }

        // BPM, level (Java line 94-95)
        model.initial_bpm = info.init_bpm;
        model.play_level = info.level;

        // Mode (Java line 96-102)
        model.mode = PlayMode::from_mode_hint(&info.mode_hint).unwrap_or(PlayMode::Beat7K);

        // LN type (Java line 103-104)
        if info.ln_type > 0 && info.ln_type <= 3 {
            model.ln_type = match info.ln_type {
                2 => LnType::ChargeNote,
                3 => LnType::HellChargeNote,
                _ => LnType::LongNote,
            };
        }

        // Key assignment table (Java line 106-119)
        let keyassign: Vec<i32> = match model.mode {
            PlayMode::Beat5K => vec![0, 1, 2, 3, 4, -1, -1, 5],
            PlayMode::Beat10K => {
                vec![0, 1, 2, 3, 4, -1, -1, 5, 6, 7, 8, 9, 10, -1, -1, 11]
            }
            _ => {
                let key_count = model.mode.key_count();
                (0..key_count as i32).collect()
            }
        };

        // Images (Java line 123-126)
        model.banner = info.banner_image.clone();
        model.back_bmp = info.back_image.clone();
        model.stage_file = info.eyecatch_image.clone();
        model.preview = info.preview_music.clone();

        // Resolution (Java line 141)
        let resolution = if info.resolution > 0 {
            info.resolution as f64 * 4.0
        } else {
            960.0
        };

        // Initialize timeline cache with y=0 entry
        let mut tlcache: BTreeMap<i32, TimeLineEntry> = BTreeMap::new();
        tlcache.insert(
            0,
            TimeLineEntry {
                time: 0.0,
                bpm: model.initial_bpm,
                stop_us: 0,
            },
        );

        // Sort BPM/STOP/Scroll events by y
        let mut bpm_events: Vec<(i32, f64)> =
            bmson.bpm_events.iter().map(|e| (e.y, e.bpm)).collect();
        bpm_events.sort_by_key(|e| e.0);

        let mut stop_events: Vec<(i32, i64)> = bmson
            .stop_events
            .iter()
            .map(|e| (e.y, e.duration))
            .collect();
        stop_events.sort_by_key(|e| e.0);

        let mut scroll_events: Vec<(i32, f64)> =
            bmson.scroll_events.iter().map(|e| (e.y, e.rate)).collect();
        scroll_events.sort_by_key(|e| e.0);

        // Merge BPM/STOP/Scroll events (Java line 152-178)
        let mut bpmpos = 0;
        let mut stoppos = 0;
        let mut scrollpos = 0;

        while bpmpos < bpm_events.len()
            || stoppos < stop_events.len()
            || scrollpos < scroll_events.len()
        {
            let bpmy = bpm_events.get(bpmpos).map_or(i32::MAX, |e| e.0);
            let stopy = stop_events.get(stoppos).map_or(i32::MAX, |e| e.0);
            let scrolly = scroll_events.get(scrollpos).map_or(i32::MAX, |e| e.0);

            if scrolly <= stopy && scrolly <= bpmy {
                // Scroll event (structure only, applied in Phase 10)
                let _tl =
                    get_or_create_timeline(&mut tlcache, scroll_events[scrollpos].0, resolution);
                scrollpos += 1;
            } else if bpmy <= stopy {
                if bpm_events[bpmpos].1 > 0.0 {
                    let entry =
                        get_or_create_timeline(&mut tlcache, bpm_events[bpmpos].0, resolution);
                    entry.bpm = bpm_events[bpmpos].1;
                }
                bpmpos += 1;
            } else if stopy != i32::MAX {
                if stop_events[stoppos].1 >= 0 {
                    let entry =
                        get_or_create_timeline(&mut tlcache, stop_events[stoppos].0, resolution);
                    let bpm = entry.bpm;
                    entry.stop_us = ((1000.0 * 1000.0 * 60.0 * 4.0 * stop_events[stoppos].1 as f64)
                        / (bpm * resolution)) as i64;
                }
                stoppos += 1;
            }
        }

        // Process sound channels → notes (Java line 186-317)
        let mut lnranges: Vec<Vec<LnRange>> = vec![Vec::new(); model.mode.key_count()];
        let ln_mode = model.ln_type;

        let mut wav_id: u16 = 0;
        for sc in &bmson.sound_channels {
            let mut notes: Vec<(i32, &crate::bmson::SoundNote)> =
                sc.notes.iter().map(|n| (n.y, n)).collect();
            notes.sort_by_key(|n| n.0);

            let mut starttime: i64 = 0;

            for (idx, &(_, n)) in notes.iter().enumerate() {
                // Find next note with y > n.y
                let mut next_y: Option<i32> = None;
                for &(y, _) in notes.iter().skip(idx + 1) {
                    if y > n.y {
                        next_y = Some(y);
                        break;
                    }
                }

                if !n.c {
                    starttime = 0;
                }

                let tl_time = get_timeline_time(&tlcache, n.y, resolution);
                let duration: i64 = if let Some(next_y) = next_y
                    && notes
                        .get(idx + 1..)
                        .is_some_and(|rest| rest.iter().any(|&(y, nn)| y == next_y && nn.c))
                {
                    let next_time = get_timeline_time(&tlcache, next_y, resolution);
                    next_time - tl_time
                } else {
                    0
                };

                let key = if n.x > 0 && (n.x as usize) <= keyassign.len() {
                    keyassign[n.x as usize - 1]
                } else {
                    -1
                };

                if key < 0 {
                    // BGM note
                    model.bg_notes.push(BgNote {
                        wav_id,
                        time_us: tl_time,
                        micro_starttime: starttime,
                        micro_duration: duration,
                    });
                } else if n.up {
                    // LN end sound definition — find matching LN and set end_wav_id
                    let lane = key as usize;
                    let section = n.y as f64 / resolution;
                    // We handle this by looking at already-placed LN notes
                    // and updating their end_wav_id if end section matches
                    for note in model.notes.iter_mut().rev() {
                        if note.lane == lane && note.is_long_note() {
                            let _end_section = (note.end_time_us as f64 - 0.5).max(0.0); // placeholder
                            // Match by end y position: compute end_y from end_time_us
                            let note_end_section =
                                find_section_for_time(&tlcache, note.end_time_us, resolution);
                            if (note_end_section - section).abs() < 0.001 {
                                note.end_wav_id = wav_id;
                                break;
                            }
                        }
                    }
                } else {
                    let lane = key as usize;
                    let section = n.y as f64 / resolution;

                    // Check if inside an existing LN
                    let inside_ln = lnranges.get(lane).is_some_and(|ranges| {
                        ranges
                            .iter()
                            .any(|r| r.start_section < section && section <= r.end_section)
                    });

                    if inside_ln {
                        // Inside LN: demote to BGM
                        model.bg_notes.push(BgNote {
                            wav_id,
                            time_us: tl_time,
                            micro_starttime: starttime,
                            micro_duration: duration,
                        });
                    } else if n.l > 0 {
                        // Long note
                        let end_y = n.y + n.l;
                        let end_time = get_timeline_time(&tlcache, end_y, resolution);
                        let end_section = end_y as f64 / resolution;

                        // Determine LN type
                        let note_ln_type = if n.t > 0 && n.t <= 3 {
                            match n.t {
                                2 => LnType::ChargeNote,
                                3 => LnType::HellChargeNote,
                                _ => LnType::LongNote,
                            }
                        } else {
                            ln_mode
                        };

                        let note = Note::long_note(
                            lane,
                            tl_time,
                            end_time,
                            wav_id,
                            wav_id, // end_wav_id defaults to same
                            note_ln_type,
                        );
                        model.notes.push(note);

                        // Track LN range
                        if let Some(ranges) = lnranges.get_mut(lane) {
                            ranges.push(LnRange {
                                start_section: section,
                                end_section,
                            });
                        }
                    } else {
                        // Normal note
                        model.notes.push(Note::normal(lane, tl_time, wav_id));
                    }
                }

                starttime += duration;
            }
            wav_id += 1;
        }

        // Key channels → invisible notes (Java line 319-334)
        for sc in &bmson.key_channels {
            let mut notes: Vec<&crate::bmson::BmsonMineNote> = sc.notes.iter().collect();
            notes.sort_by_key(|n| n.y);

            for n in notes {
                let key = if n.x > 0 && (n.x as usize) <= keyassign.len() {
                    keyassign[n.x as usize - 1]
                } else {
                    -1
                };
                if key >= 0 {
                    let tl_time = get_timeline_time(&tlcache, n.y, resolution);
                    model
                        .notes
                        .push(Note::invisible(key as usize, tl_time, wav_id));
                }
            }
            wav_id += 1;
        }

        // Mine channels → mine notes (Java line 335-368)
        for sc in &bmson.mine_channels {
            let mut notes: Vec<&crate::bmson::BmsonMineNote> = sc.notes.iter().collect();
            notes.sort_by_key(|n| n.y);

            for n in notes {
                let key = if n.x > 0 && (n.x as usize) <= keyassign.len() {
                    keyassign[n.x as usize - 1]
                } else {
                    -1
                };
                if key >= 0 {
                    let lane = key as usize;
                    let section = n.y as f64 / resolution;

                    // Check if inside LN
                    let inside_ln = lnranges.get(lane).is_some_and(|ranges| {
                        ranges
                            .iter()
                            .any(|r| r.start_section < section && section <= r.end_section)
                    });

                    if !inside_ln {
                        let tl_time = get_timeline_time(&tlcache, n.y, resolution);
                        model
                            .notes
                            .push(Note::mine(lane, tl_time, wav_id, n.damage as i32));
                    }
                }
            }
            wav_id += 1;
        }

        // Post-processing: sort notes by (time_us, lane)
        model
            .notes
            .sort_by(|a, b| a.time_us.cmp(&b.time_us).then_with(|| a.lane.cmp(&b.lane)));

        // Sort background notes by time
        model.bg_notes.sort_by_key(|n| n.time_us);

        // Deduplicate: same (lane, time_us) → keep highest priority
        model.notes.dedup_by(|b, a| {
            if a.lane == b.lane && a.time_us == b.time_us {
                if note_priority(b) > note_priority(a) {
                    std::mem::swap(a, b);
                }
                true
            } else {
                false
            }
        });

        // Build BPM changes from tlcache
        let mut prev_bpm = model.initial_bpm;
        for (&_y, entry) in &tlcache {
            if (entry.bpm - prev_bpm).abs() > 0.001 {
                model.bpm_changes.push(BpmChange {
                    time_us: entry.time as i64,
                    bpm: entry.bpm,
                });
                prev_bpm = entry.bpm;
            }
        }

        // Build stop events from tlcache
        for (&_y, entry) in &tlcache {
            if entry.stop_us > 0 {
                model.stop_events.push(StopEvent {
                    time_us: entry.time as i64,
                    duration_ticks: 0, // bmson uses duration in pulses, not BMS ticks
                    duration_us: entry.stop_us,
                });
            }
        }

        // Build timelines
        let mut seen_times: Vec<i64> = model.notes.iter().map(|n| n.time_us).collect();
        seen_times.sort();
        seen_times.dedup();
        for &t in &seen_times {
            let bpm = bpm_at_time(t, model.initial_bpm, &model.bpm_changes);
            model.timelines.push(TimeLine {
                time_us: t,
                measure: 0,
                position: 0.0,
                bpm,
            });
        }

        // Compute total_time_us
        if let Some(last) = tlcache.values().last() {
            model.total_time_us = (last.time + last.stop_us as f64) as i64;
        }
        // Also consider the last note's time
        if let Some(last_note) = model.notes.last() {
            let end = if last_note.is_long_note() {
                last_note.end_time_us
            } else {
                last_note.time_us
            };
            if end > model.total_time_us {
                model.total_time_us = end;
            }
        }

        Ok(())
    }
}

/// Get or create a timeline entry, computing time from previous entries
fn get_or_create_timeline(
    tlcache: &mut BTreeMap<i32, TimeLineEntry>,
    y: i32,
    resolution: f64,
) -> &mut TimeLineEntry {
    if tlcache.contains_key(&y) {
        return tlcache.get_mut(&y).unwrap();
    }

    // Find the lower entry
    let (&prev_y, prev) = tlcache.range(..y).next_back().unwrap();
    let bpm = prev.bpm;
    let time = prev.time
        + prev.stop_us as f64
        + (240_000.0 * 1000.0 * ((y - prev_y) as f64 / resolution)) / bpm;

    tlcache.insert(
        y,
        TimeLineEntry {
            time,
            bpm,
            stop_us: 0,
        },
    );
    tlcache.get_mut(&y).unwrap()
}

/// Get the time (μs) for a given y position from the timeline cache
fn get_timeline_time(tlcache: &BTreeMap<i32, TimeLineEntry>, y: i32, resolution: f64) -> i64 {
    if let Some(entry) = tlcache.get(&y) {
        return entry.time as i64;
    }

    let (&prev_y, prev) = tlcache.range(..=y).next_back().unwrap();
    let time = prev.time
        + prev.stop_us as f64
        + (240_000.0 * 1000.0 * ((y - prev_y) as f64 / resolution)) / prev.bpm;
    time as i64
}

/// Find the section (y/resolution) for a given time_us by searching the tlcache
fn find_section_for_time(
    tlcache: &BTreeMap<i32, TimeLineEntry>,
    time_us: i64,
    resolution: f64,
) -> f64 {
    let time = time_us as f64;

    // Find the entry just before this time
    let mut best_y = 0;
    let mut best_entry_time = 0.0;
    let mut best_bpm = 130.0;

    for (&y, entry) in tlcache {
        if entry.time <= time {
            best_y = y;
            best_entry_time = entry.time + entry.stop_us as f64;
            best_bpm = entry.bpm;
        } else {
            break;
        }
    }

    // Compute the y position from the remaining time
    let remaining_time = time - best_entry_time;
    let remaining_y = (remaining_time * best_bpm * resolution) / (240_000.0 * 1000.0);
    (best_y as f64 + remaining_y) / resolution
}

/// Note priority for deduplication: Normal/Invisible > LN > Mine
fn note_priority(n: &Note) -> u8 {
    match n.note_type {
        NoteType::Normal | NoteType::Invisible => 2,
        NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote => 1,
        NoteType::Mine => 0,
    }
}

/// Find BPM at a given time
fn bpm_at_time(time_us: i64, initial_bpm: f64, bpm_changes: &[BpmChange]) -> f64 {
    let mut bpm = initial_bpm;
    for change in bpm_changes {
        if change.time_us <= time_us {
            bpm = change.bpm;
        } else {
            break;
        }
    }
    bpm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_minimal_bmson() {
        let json = r#"{
            "version": "1.0.0",
            "info": {
                "title": "Test Song",
                "artist": "Test Artist",
                "init_bpm": 120.0,
                "mode_hint": "beat-7k",
                "resolution": 240
            },
            "sound_channels": [
                {
                    "name": "kick.wav",
                    "notes": [
                        {"x": 1, "y": 0, "l": 0, "c": false, "t": 0, "up": false},
                        {"x": 2, "y": 240, "l": 0, "c": false, "t": 0, "up": false}
                    ]
                }
            ]
        }"#;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bmson");
        std::fs::write(&path, json).unwrap();

        let model = BmsonDecoder::decode(&path).unwrap();
        assert_eq!(model.title, "Test Song");
        assert_eq!(model.artist, "Test Artist");
        assert_eq!(model.initial_bpm, 120.0);
        assert_eq!(model.mode, PlayMode::Beat7K);
        assert_eq!(model.total_notes(), 2);
    }

    #[test]
    fn test_decode_bmson_longnote() {
        let json = r#"{
            "info": {
                "title": "LN Test",
                "init_bpm": 120.0,
                "mode_hint": "beat-7k",
                "resolution": 240
            },
            "sound_channels": [
                {
                    "name": "sound.wav",
                    "notes": [
                        {"x": 1, "y": 0, "l": 480, "c": false, "t": 0, "up": false}
                    ]
                }
            ]
        }"#;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bmson");
        std::fs::write(&path, json).unwrap();

        let model = BmsonDecoder::decode(&path).unwrap();
        assert_eq!(model.total_notes(), 1);
        let ln = &model.notes[0];
        assert!(ln.is_long_note());
        assert!(ln.end_time_us > ln.time_us);
    }

    #[test]
    fn test_decode_bmson_bpm_change() {
        let json = r#"{
            "info": {
                "title": "BPM Test",
                "init_bpm": 120.0,
                "resolution": 240
            },
            "bpm_events": [
                {"y": 960, "bpm": 180.0}
            ],
            "sound_channels": [
                {
                    "name": "sound.wav",
                    "notes": [
                        {"x": 1, "y": 0, "l": 0, "c": false, "t": 0, "up": false},
                        {"x": 1, "y": 1440, "l": 0, "c": false, "t": 0, "up": false}
                    ]
                }
            ]
        }"#;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bmson");
        std::fs::write(&path, json).unwrap();

        let model = BmsonDecoder::decode(&path).unwrap();
        assert_eq!(model.bpm_changes.len(), 1);
        assert_eq!(model.bpm_changes[0].bpm, 180.0);
    }

    #[test]
    fn test_decode_bmson_5k_keyassign() {
        let json = r#"{
            "info": {
                "title": "5K Test",
                "init_bpm": 120.0,
                "mode_hint": "beat-5k",
                "resolution": 240
            },
            "sound_channels": [
                {
                    "name": "sound.wav",
                    "notes": [
                        {"x": 8, "y": 0, "l": 0, "c": false, "t": 0, "up": false}
                    ]
                }
            ]
        }"#;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bmson");
        std::fs::write(&path, json).unwrap();

        let model = BmsonDecoder::decode(&path).unwrap();
        assert_eq!(model.mode, PlayMode::Beat5K);
        // x=8 → keyassign[7] = 5 (scratch)
        assert_eq!(model.notes[0].lane, 5);
    }

    #[test]
    fn test_mode_hint_parsing() {
        assert_eq!(PlayMode::from_mode_hint("beat-7k"), Some(PlayMode::Beat7K));
        assert_eq!(PlayMode::from_mode_hint("beat-5k"), Some(PlayMode::Beat5K));
        assert_eq!(PlayMode::from_mode_hint("popn-9k"), Some(PlayMode::PopN9K));
        assert_eq!(PlayMode::from_mode_hint("unknown"), None);
    }
}

// osu!mania format decoder
// Converts .osu beatmap files (mode=3, mania) to BmsModel
// Reference: Java OSUDecoder.java (394 lines)

use std::collections::BTreeMap;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Result, bail};

use crate::mode::PlayMode;
use crate::model::{BmsModel, TotalType};
use crate::note::{BgNote, Note};
use crate::osu::Osu;
use crate::timeline::{BpmChange, TimeLine};

/// osu!mania decoder
pub struct OsuDecoder;

impl OsuDecoder {
    /// Decode an .osu file into a BmsModel.
    /// Only osu!mania (mode=3) is supported.
    pub fn decode(path: &Path) -> Result<BmsModel> {
        let raw_bytes = std::fs::read(path)?;

        // Compute hashes
        let md5 = format!("{:x}", md5::compute(&raw_bytes));
        let sha256 = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&raw_bytes);
            format!("{:x}", hasher.finalize())
        };

        let reader = BufReader::new(&raw_bytes[..]);
        let osu = Osu::parse(reader);

        if osu.timing_points.is_empty() || osu.hit_objects.is_empty() {
            bail!("osu file has no timing points or hit objects");
        }
        if osu.general.mode != 3 {
            bail!(
                "only osu!mania (mode=3) is supported, got mode={}",
                osu.general.mode
            );
        }

        let keymode = osu.difficulty.circle_size as i32;
        let (mode, mapping) = keymode_to_mode_and_mapping(keymode)?;

        let mut model = BmsModel {
            md5,
            sha256,
            mode,
            title: osu.metadata.title.clone(),
            subtitle: format!("[{}]", osu.metadata.version),
            artist: osu.metadata.artist.clone(),
            sub_artist: osu.metadata.creator.clone(),
            genre: format!("{}K", keymode),
            judge_rank: 3,
            total: 0.0,
            total_type: TotalType::Bms,
            preview: osu.general.audio_filename.clone(),
            ..Default::default()
        };

        let offset = 38; // Java adds 38ms offset to all timings

        // Separate timing points into BPM changes and scroll velocity changes
        let mut bpm_points: Vec<crate::osu::TimingPoints> = Vec::new();
        let mut sv_points: Vec<crate::osu::TimingPoints> = Vec::new();

        for (i, point) in osu.timing_points.iter().enumerate() {
            let mut p = point.clone();
            p.time += offset as f32;

            if p.uninherited {
                bpm_points.push(p.clone());

                // Create a default SV entry for each BPM point
                let mut sv = p.clone();
                sv.beat_length = -100.0;
                sv.uninherited = false;

                // Only add if the next point doesn't share the same time
                let add_sv = if i + 1 < osu.timing_points.len() {
                    (osu.timing_points[i + 1].time + offset as f32) != p.time
                } else {
                    true
                };
                if add_sv {
                    sv_points.push(sv);
                }
            } else {
                // Inherited timing point (SV change)
                if let Some(last_sv) = sv_points.last_mut()
                    && (last_sv.time - p.time).abs() < 0.001
                {
                    last_sv.beat_length = p.beat_length;
                    continue;
                }
                sv_points.push(p);
            }
        }

        // Set initial BPM
        if !bpm_points.is_empty() {
            model.initial_bpm = bpm_to_f64(bpm_points[0].beat_length);
        }

        // Add BGM note at time 0 (audio file)
        model.bg_notes.push(BgNote {
            wav_id: 0,
            time_us: 0,
            micro_starttime: 0,
            micro_duration: 0,
        });

        // Process BPM changes → BpmChange events
        for point in &bpm_points {
            let time_us = (point.time as i64) * 1000;
            let bpm = bpm_to_f64(point.beat_length);
            model.bpm_changes.push(BpmChange { time_us, bpm });
        }

        // Process events (background/video/sample sounds)
        for event in &osu.events {
            match event.event_type.as_str() {
                "0" => {
                    // Background image
                    if let Some(param) = event.event_params.first() {
                        model.back_bmp = param.clone();
                        model.stage_file = param.clone();
                    }
                }
                "5" | "Sample" => {
                    // Background sound
                    if event.event_params.len() > 1 {
                        let name = event.event_params[1].replace('"', "");
                        let time_ms = event.start_time + offset;
                        model.bg_notes.push(BgNote {
                            wav_id: 0,
                            time_us: time_ms as i64 * 1000,
                            micro_starttime: 0,
                            micro_duration: 0,
                        });
                        // Store wav filename for reference
                        let wav_id = model.wav_defs.len() as u16;
                        model
                            .wav_defs
                            .insert(wav_id, path.parent().unwrap_or(Path::new(".")).join(name));
                    }
                }
                _ => {}
            }
        }

        // Process hit objects → Notes
        for hit in &osu.hit_objects {
            if hit.time < 0 {
                continue;
            }
            let time_ms = hit.time + offset;
            let time_us = time_ms as i64 * 1000;

            // Map x position to column index
            let column_idx = ((hit.x as f32 * keymode as f32 / 512.0).floor() as i32)
                .clamp(0, keymode - 1) as usize;

            // Map column to lane using keymode mapping
            if column_idx >= mapping.len() {
                continue;
            }
            let lane = mapping[column_idx];
            if lane < 0 {
                continue;
            }
            let lane = lane as usize;

            let is_hold = (hit.hit_type & 0x80) > 0;

            if is_hold {
                // Mania hold note → LongNote
                let tail_time_ms = hit
                    .object_params
                    .first()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(time_ms)
                    + offset;

                if tail_time_ms <= time_ms {
                    // Degenerate hold → normal note
                    model.notes.push(Note::normal(lane, time_us, 0));
                } else {
                    let tail_time_us = tail_time_ms as i64 * 1000;
                    model.notes.push(Note::long_note(
                        lane,
                        time_us,
                        tail_time_us,
                        0,
                        0,
                        model.ln_type,
                    ));
                }
            } else {
                // Normal note
                model.notes.push(Note::normal(lane, time_us, 0));
            }
        }

        // Sort notes by time, then lane
        model
            .notes
            .sort_by(|a, b| a.time_us.cmp(&b.time_us).then_with(|| a.lane.cmp(&b.lane)));

        // Build timelines from note times
        let mut tl_times: BTreeMap<i64, ()> = BTreeMap::new();
        for note in &model.notes {
            tl_times.insert(note.time_us, ());
            if note.is_long_note() && note.end_time_us > 0 {
                tl_times.insert(note.end_time_us, ());
            }
        }
        for bg in &model.bg_notes {
            tl_times.insert(bg.time_us, ());
        }
        for change in &model.bpm_changes {
            tl_times.insert(change.time_us, ());
        }

        for &time_us in tl_times.keys() {
            let bpm = bpm_at_time(time_us, &model);
            let scroll = sv_at_time(time_us, &sv_points);
            model.timelines.push(TimeLine {
                time_us,
                measure: 0,
                position: 0.0,
                bpm,
                scroll,
            });
        }

        // Set total time
        let last_note_time = model
            .notes
            .iter()
            .map(|n| {
                if n.is_long_note() && n.end_time_us > 0 {
                    n.end_time_us
                } else {
                    n.time_us
                }
            })
            .max()
            .unwrap_or(0);
        model.total_time_us = last_note_time;

        // Store audio file as WAV definition 0
        model.wav_defs.insert(
            0,
            path.parent()
                .unwrap_or(Path::new("."))
                .join(&osu.general.audio_filename),
        );

        Ok(model)
    }
}

/// Map osu!mania keymode to PlayMode and lane mapping.
/// Returns (PlayMode, mapping) where mapping[column] = lane (-1 = unused).
fn keymode_to_mode_and_mapping(keymode: i32) -> Result<(PlayMode, Vec<i32>)> {
    match keymode {
        4 => Ok((PlayMode::Beat7K, vec![0, 2, 4, 6, -1, -1, -1, -1])),
        5 => Ok((PlayMode::Beat5K, vec![0, 1, 2, 3, 4, -1])),
        6 => Ok((PlayMode::Beat7K, vec![0, 1, 2, 4, 5, 6, -1, -1])),
        7 => Ok((PlayMode::Beat7K, vec![0, 1, 2, 3, 4, 5, 6, -1])),
        8 => Ok((PlayMode::Beat7K, vec![7, 0, 1, 2, 3, 4, 5, 6])),
        9 => Ok((PlayMode::PopN9K, vec![0, 1, 2, 3, 4, 5, 6, 7, 8])),
        10 => Ok((
            PlayMode::Beat10K,
            vec![0, 1, 2, 3, 4, 6, 7, 8, 9, 10, -1, -1],
        )),
        12 => Ok((
            PlayMode::Beat10K,
            vec![5, 0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11],
        )),
        14 => Ok((
            PlayMode::Beat14K,
            vec![0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, -1, -1],
        )),
        16 => Ok((
            PlayMode::Beat14K,
            vec![7, 0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15],
        )),
        _ => bail!("unsupported keymode: {}", keymode),
    }
}

/// Convert beat_length (ms per beat) to BPM
fn bpm_to_f64(beat_length: f32) -> f64 {
    if beat_length.abs() < f32::EPSILON {
        return 0.0;
    }
    1.0 / beat_length as f64 * 1000.0 * 60.0
}

/// Get BPM at a given time
fn bpm_at_time(time_us: i64, model: &BmsModel) -> f64 {
    let mut bpm = model.initial_bpm;
    for change in &model.bpm_changes {
        if change.time_us <= time_us {
            bpm = change.bpm;
        } else {
            break;
        }
    }
    bpm
}

/// Get scroll velocity at a given time from SV points
fn sv_at_time(time_us: i64, sv_points: &[crate::osu::TimingPoints]) -> f64 {
    if sv_points.is_empty() {
        return 1.0;
    }
    let time_ms = time_us as f32 / 1000.0;
    if sv_points[0].time > time_ms {
        return 1.0;
    }

    let mut scroll = 1.0;
    for sv in sv_points {
        if sv.time <= time_ms {
            scroll = 100.0 / (-sv.beat_length as f64);
        } else {
            break;
        }
    }
    scroll
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osu::Osu;
    use std::io::Cursor;

    fn make_osu_content(keymode: i32) -> String {
        format!(
            "osu file format v14\n\
            \n\
            [General]\n\
            AudioFilename: audio.mp3\n\
            Mode: 3\n\
            \n\
            [Metadata]\n\
            Title:Test Song\n\
            Artist:Test Artist\n\
            Creator:Test Creator\n\
            Version:Normal\n\
            \n\
            [Difficulty]\n\
            CircleSize:{keymode}\n\
            \n\
            [TimingPoints]\n\
            0,500,4,1,0,100,1,0\n\
            \n\
            [HitObjects]\n\
            64,192,1000,1,0,0:0:0:0:\n\
            192,192,2000,1,0,0:0:0:0:\n\
            320,192,3000,128,0,4000:0:0:0:0:\n"
        )
    }

    #[test]
    fn test_osu_parse_minimal() {
        let content = make_osu_content(4);
        let reader = Cursor::new(content.as_bytes());
        let osu = Osu::parse(reader);
        assert_eq!(osu.general.mode, 3);
        assert_eq!(osu.difficulty.circle_size as i32, 4);
        assert_eq!(osu.timing_points.len(), 1);
        assert_eq!(osu.hit_objects.len(), 3);
    }

    #[test]
    fn test_osu_decode_4k() {
        let content = make_osu_content(4);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.osu");
        std::fs::write(&path, content).unwrap();

        let model = OsuDecoder::decode(&path).unwrap();
        assert_eq!(model.mode, PlayMode::Beat7K);
        assert_eq!(model.title, "Test Song");
        assert_eq!(model.artist, "Test Artist");
        assert!(!model.notes.is_empty());
    }

    #[test]
    fn test_osu_decode_7k() {
        let content = make_osu_content(7);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.osu");
        std::fs::write(&path, content).unwrap();

        let model = OsuDecoder::decode(&path).unwrap();
        assert_eq!(model.mode, PlayMode::Beat7K);
    }

    #[test]
    fn test_osu_hold_note() {
        let content = make_osu_content(4);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.osu");
        std::fs::write(&path, content).unwrap();

        let model = OsuDecoder::decode(&path).unwrap();
        let lns: Vec<&Note> = model.notes.iter().filter(|n| n.is_long_note()).collect();
        assert_eq!(lns.len(), 1, "should have 1 hold note");
        assert!(
            lns[0].end_time_us > lns[0].time_us,
            "hold end must be after start"
        );
    }

    #[test]
    fn test_osu_non_mania_rejected() {
        let content = "osu file format v14\n\
            \n\
            [General]\n\
            AudioFilename: audio.mp3\n\
            Mode: 0\n\
            \n\
            [Difficulty]\n\
            CircleSize:4\n\
            \n\
            [TimingPoints]\n\
            0,500,4,1,0,100,1,0\n\
            \n\
            [HitObjects]\n\
            64,192,1000,1,0,0:0:0:0:\n";

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.osu");
        std::fs::write(&path, content).unwrap();

        let result = OsuDecoder::decode(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_bpm_conversion() {
        // 500ms per beat = 120 BPM
        let bpm = bpm_to_f64(500.0);
        assert!((bpm - 120.0).abs() < 0.01);
    }
}

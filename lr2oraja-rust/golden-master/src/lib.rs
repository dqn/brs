// Golden master test infrastructure: Java fixture comparison harness

pub mod audio_fixtures;
pub mod autoplay_fixtures;
pub mod database_fixtures;
pub mod e2e_helpers;
pub mod judge_fixtures;
pub mod pattern_fixtures;
pub mod render_snapshot;
pub mod replay_e2e_fixtures;
pub mod rule_fixtures;
pub mod skin_fixtures;

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Java fixture root structure
#[derive(Debug, Deserialize)]
pub struct Fixture {
    pub metadata: FixtureMetadata,
    pub hashes: FixtureHashes,
    pub statistics: FixtureStatistics,
    #[serde(default)]
    pub timelines: Vec<FixtureTimeline>,
    pub notes: Vec<FixtureNote>,
    pub bpm_changes: Vec<FixtureBpmChange>,
    pub stop_events: Vec<FixtureStopEvent>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureMetadata {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub sub_artist: String,
    pub genre: String,
    pub initial_bpm: f64,
    pub judge_rank: i32,
    pub total: f64,
    pub player: i32,
    pub mode: String,
    pub mode_key_count: usize,
    pub ln_type: i32,
    pub banner: String,
    pub stagefile: String,
    pub backbmp: String,
    pub preview: String,
}

#[derive(Debug, Deserialize)]
pub struct FixtureHashes {
    pub md5: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
pub struct FixtureStatistics {
    pub total_notes: usize,
    pub total_notes_mine: usize,
    pub min_bpm: f64,
    pub max_bpm: f64,
    pub timeline_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct FixtureTimeline {
    pub time_us: i64,
    pub bpm: f64,
    pub stop_us: i64,
    #[serde(default)]
    pub notes: Vec<FixtureNote>,
    #[serde(default)]
    pub hidden_notes: Vec<FixtureNote>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureNote {
    pub lane: usize,
    pub time_us: i64,
    /// Java wav_id is a wavlist index (0-based), different from Rust base36 value
    pub wav_id: i32,
    #[serde(rename = "type")]
    pub note_type: String,
    pub end_time_us: Option<i64>,
    /// May be -2 for undefined in Java
    pub end_wav_id: Option<i32>,
    pub damage: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureBpmChange {
    pub time_us: i64,
    pub bpm: f64,
}

#[derive(Debug, Deserialize)]
pub struct FixtureStopEvent {
    pub time_us: i64,
    pub duration_us: i64,
}

/// Load a fixture JSON file
pub fn load_fixture(path: &Path) -> Result<Fixture> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read fixture: {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse fixture: {}", path.display()))
}

/// Map Java mode hint string to Rust PlayMode
pub fn mode_hint_to_play_mode(hint: &str) -> Option<bms_model::PlayMode> {
    match hint {
        "beat-5k" => Some(bms_model::PlayMode::Beat5K),
        "beat-7k" => Some(bms_model::PlayMode::Beat7K),
        "beat-10k" => Some(bms_model::PlayMode::Beat10K),
        "beat-14k" => Some(bms_model::PlayMode::Beat14K),
        "popn-5k" => Some(bms_model::PlayMode::PopN5K),
        "popn-9k" => Some(bms_model::PlayMode::PopN9K),
        "keyboard-24k" => Some(bms_model::PlayMode::Keyboard24K),
        "keyboard-24k-double" => Some(bms_model::PlayMode::Keyboard24KDouble),
        _ => None,
    }
}

/// Map Java note type string to Rust NoteType
pub fn fixture_note_type_to_rust(type_str: &str) -> Option<bms_model::NoteType> {
    match type_str {
        "Normal" => Some(bms_model::NoteType::Normal),
        "LongNote" => Some(bms_model::NoteType::LongNote),
        "LongNoteUndefined" => Some(bms_model::NoteType::LongNote),
        "ChargeNote" => Some(bms_model::NoteType::ChargeNote),
        "HellChargeNote" => Some(bms_model::NoteType::HellChargeNote),
        "Mine" => Some(bms_model::NoteType::Mine),
        "Invisible" => Some(bms_model::NoteType::Invisible),
        _ => None,
    }
}

/// Compare a Rust BmsModel against a Java fixture.
/// Returns a list of differences found.
pub fn compare_model(model: &bms_model::BmsModel, fixture: &Fixture) -> Vec<String> {
    let mut diffs = Vec::new();

    // Metadata
    if model.title != fixture.metadata.title {
        diffs.push(format!(
            "title: rust={:?} java={:?}",
            model.title, fixture.metadata.title
        ));
    }
    if model.subtitle != fixture.metadata.subtitle {
        diffs.push(format!(
            "subtitle: rust={:?} java={:?}",
            model.subtitle, fixture.metadata.subtitle
        ));
    }
    if model.artist != fixture.metadata.artist {
        diffs.push(format!(
            "artist: rust={:?} java={:?}",
            model.artist, fixture.metadata.artist
        ));
    }
    if model.sub_artist != fixture.metadata.sub_artist {
        diffs.push(format!(
            "sub_artist: rust={:?} java={:?}",
            model.sub_artist, fixture.metadata.sub_artist
        ));
    }
    if model.genre != fixture.metadata.genre {
        diffs.push(format!(
            "genre: rust={:?} java={:?}",
            model.genre, fixture.metadata.genre
        ));
    }
    if (model.initial_bpm - fixture.metadata.initial_bpm).abs() > 0.001 {
        diffs.push(format!(
            "initial_bpm: rust={} java={}",
            model.initial_bpm, fixture.metadata.initial_bpm
        ));
    }
    if model.judge_rank_raw != fixture.metadata.judge_rank {
        diffs.push(format!(
            "judge_rank: rust={} java={}",
            model.judge_rank_raw, fixture.metadata.judge_rank
        ));
    }
    if (model.total - fixture.metadata.total).abs() > 0.001 {
        diffs.push(format!(
            "total: rust={} java={}",
            model.total, fixture.metadata.total
        ));
    }
    if fixture.metadata.player > 0 && model.player != fixture.metadata.player {
        diffs.push(format!(
            "player: rust={} java={}",
            model.player, fixture.metadata.player
        ));
    }
    if model.mode.key_count() != fixture.metadata.mode_key_count {
        diffs.push(format!(
            "mode_key_count: rust={} java={}",
            model.mode.key_count(),
            fixture.metadata.mode_key_count
        ));
    }
    if fixture.metadata.ln_type > 0 && model.ln_type as i32 != fixture.metadata.ln_type {
        diffs.push(format!(
            "ln_type: rust={} java={}",
            model.ln_type as i32, fixture.metadata.ln_type
        ));
    }
    if model.banner != fixture.metadata.banner {
        diffs.push(format!(
            "banner: rust={:?} java={:?}",
            model.banner, fixture.metadata.banner
        ));
    }
    if model.stage_file != fixture.metadata.stagefile {
        diffs.push(format!(
            "stagefile: rust={:?} java={:?}",
            model.stage_file, fixture.metadata.stagefile
        ));
    }
    if model.back_bmp != fixture.metadata.backbmp {
        diffs.push(format!(
            "backbmp: rust={:?} java={:?}",
            model.back_bmp, fixture.metadata.backbmp
        ));
    }
    if model.preview != fixture.metadata.preview {
        diffs.push(format!(
            "preview: rust={:?} java={:?}",
            model.preview, fixture.metadata.preview
        ));
    }

    // Play mode
    if let Some(expected_mode) = mode_hint_to_play_mode(&fixture.metadata.mode)
        && model.mode != expected_mode
    {
        diffs.push(format!(
            "mode: rust={:?} java={:?} ({})",
            model.mode, expected_mode, fixture.metadata.mode
        ));
    }

    // Hashes
    if model.md5 != fixture.hashes.md5 {
        diffs.push(format!(
            "md5: rust={} java={}",
            model.md5, fixture.hashes.md5
        ));
    }
    if model.sha256 != fixture.hashes.sha256 {
        diffs.push(format!(
            "sha256: rust={} java={}",
            model.sha256, fixture.hashes.sha256
        ));
    }

    // Statistics
    if model.total_notes() != fixture.statistics.total_notes {
        diffs.push(format!(
            "total_notes: rust={} java={}",
            model.total_notes(),
            fixture.statistics.total_notes
        ));
    }

    let rust_mines = model
        .notes
        .iter()
        .filter(|n| n.note_type == bms_model::NoteType::Mine)
        .count();
    if rust_mines != fixture.statistics.total_notes_mine {
        diffs.push(format!(
            "mine_notes: rust={} java={}",
            rust_mines, fixture.statistics.total_notes_mine
        ));
    }

    if (model.min_bpm() - fixture.statistics.min_bpm).abs() > 0.001 {
        diffs.push(format!(
            "min_bpm: rust={} java={}",
            model.min_bpm(),
            fixture.statistics.min_bpm
        ));
    }
    if (model.max_bpm() - fixture.statistics.max_bpm).abs() > 0.001 {
        diffs.push(format!(
            "max_bpm: rust={} java={}",
            model.max_bpm(),
            fixture.statistics.max_bpm
        ));
    }
    if fixture.statistics.timeline_count != fixture.timelines.len() {
        diffs.push(format!(
            "timeline_count(fixture_consistency): statistics={} timelines={}",
            fixture.statistics.timeline_count,
            fixture.timelines.len()
        ));
    }
    if !fixture.timelines.is_empty() {
        // Java timeline_count/getAllTimeLines include empty measure boundaries.
        // Rust model.timelines intentionally tracks note-bearing times only.
        let java_note_timelines: Vec<&FixtureTimeline> = fixture
            .timelines
            .iter()
            .filter(|timeline| {
                timeline
                    .notes
                    .iter()
                    .any(|note| note.note_type != "LongNoteEnd")
                    || !timeline.hidden_notes.is_empty()
            })
            .collect();

        if model.timelines.len() != java_note_timelines.len() {
            diffs.push(format!(
                "timeline_count(note_timelines): rust={} java={}",
                model.timelines.len(),
                java_note_timelines.len()
            ));
        }

        let mut rust_notes_by_time: HashMap<i64, usize> = HashMap::new();
        for note in &model.notes {
            *rust_notes_by_time.entry(note.time_us).or_insert(0) += 1;
        }

        let min_len = model.timelines.len().min(java_note_timelines.len());
        for (i, (rt, &ft)) in model
            .timelines
            .iter()
            .zip(java_note_timelines.iter())
            .enumerate()
            .take(min_len)
        {
            if (rt.time_us - ft.time_us).abs() > 2 {
                diffs.push(format!(
                    "timeline[{}] time_us: rust={} java={}",
                    i, rt.time_us, ft.time_us
                ));
            }
            if (rt.bpm - ft.bpm).abs() > 0.001 {
                diffs.push(format!(
                    "timeline[{}] bpm: rust={} java={}",
                    i, rt.bpm, ft.bpm
                ));
            }

            let java_note_count = ft
                .notes
                .iter()
                .filter(|note| note.note_type != "LongNoteEnd")
                .count()
                + ft.hidden_notes.len();
            let rust_note_count = rust_notes_by_time.get(&rt.time_us).copied().unwrap_or(0);
            if rust_note_count != java_note_count {
                diffs.push(format!(
                    "timeline[{}] note_count: rust={} java={}",
                    i, rust_note_count, java_note_count
                ));
            }
        }
    }

    // Notes comparison (flat list, excluding LN ends which Java filters out)
    let rust_notes: Vec<&bms_model::Note> = model
        .notes
        .iter()
        .filter(|_n| {
            // Skip invisible notes for comparison with fixture flat list
            // Java fixture includes invisible in flat notes, so include them
            true
        })
        .collect();

    let fixture_notes = &fixture.notes;

    if rust_notes.len() != fixture_notes.len() {
        diffs.push(format!(
            "note_count: rust={} java={}",
            rust_notes.len(),
            fixture_notes.len()
        ));
    }

    // Compare notes by (lane, time_us) pairs
    let min_len = rust_notes.len().min(fixture_notes.len());
    for i in 0..min_len {
        let rn = rust_notes[i];
        let fn_ = &fixture_notes[i];

        if rn.lane != fn_.lane {
            diffs.push(format!(
                "note[{}] lane: rust={} java={}",
                i, rn.lane, fn_.lane
            ));
        }

        // Allow ±2μs tolerance for floating-point rounding differences
        let time_diff = (rn.time_us - fn_.time_us).abs();
        if time_diff > 2 {
            diffs.push(format!(
                "note[{}] time_us: rust={} java={} (diff={})",
                i, rn.time_us, fn_.time_us, time_diff
            ));
        }

        // wav_id comparison skipped: Java uses wavlist index (0-based),
        // Rust uses base36 value directly. Semantics differ by design.

        if let Some(expected_type) = fixture_note_type_to_rust(&fn_.note_type)
            && rn.note_type != expected_type
        {
            diffs.push(format!(
                "note[{}] type: rust={:?} java={} (expected {:?})",
                i, rn.note_type, fn_.note_type, expected_type
            ));
        }
        if let Some(damage) = fn_.damage
            && rn.note_type == bms_model::NoteType::Mine
            && (rn.damage as f64 - damage).abs() > f64::EPSILON
        {
            diffs.push(format!(
                "note[{}] damage: rust={} java={}",
                i, rn.damage, damage
            ));
        }

        // LN end time
        if let Some(end_time) = fn_.end_time_us
            && rn.is_long_note()
        {
            let diff = (rn.end_time_us - end_time).abs();
            if diff > 2 {
                diffs.push(format!(
                    "note[{}] end_time_us: rust={} java={} (diff={})",
                    i, rn.end_time_us, end_time, diff
                ));
            }
        }
    }

    // BPM changes
    if model.bpm_changes.len() != fixture.bpm_changes.len() {
        diffs.push(format!(
            "bpm_change_count: rust={} java={}",
            model.bpm_changes.len(),
            fixture.bpm_changes.len()
        ));
    } else {
        for (i, (rc, fc)) in model
            .bpm_changes
            .iter()
            .zip(fixture.bpm_changes.iter())
            .enumerate()
        {
            if (rc.time_us - fc.time_us).abs() > 2 {
                diffs.push(format!(
                    "bpm_change[{}] time_us: rust={} java={}",
                    i, rc.time_us, fc.time_us
                ));
            }
            if (rc.bpm - fc.bpm).abs() > 0.001 {
                diffs.push(format!(
                    "bpm_change[{}] bpm: rust={} java={}",
                    i, rc.bpm, fc.bpm
                ));
            }
        }
    }

    // Stop events
    if model.stop_events.len() != fixture.stop_events.len() {
        diffs.push(format!(
            "stop_event_count: rust={} java={}",
            model.stop_events.len(),
            fixture.stop_events.len()
        ));
    } else {
        for (i, (rs, fs)) in model
            .stop_events
            .iter()
            .zip(fixture.stop_events.iter())
            .enumerate()
        {
            if (rs.time_us - fs.time_us).abs() > 2 {
                diffs.push(format!(
                    "stop_event[{}] time_us: rust={} java={}",
                    i, rs.time_us, fs.time_us
                ));
            }
            if (rs.duration_us - fs.duration_us).abs() > 2 {
                diffs.push(format!(
                    "stop_event[{}] duration_us: rust={} java={}",
                    i, rs.duration_us, fs.duration_us
                ));
            }
        }
    }

    diffs
}

/// Compare a bmson-decoded Rust BmsModel against a Java fixture.
/// Unlike BMS, bmson wav_id has the same semantics (0-based channel index)
/// so wav_id comparison is enabled.
pub fn compare_model_bmson(model: &bms_model::BmsModel, fixture: &Fixture) -> Vec<String> {
    let mut diffs = compare_model(model, fixture);

    // Additional wav_id comparison (bmson uses same 0-based index in both Java and Rust)
    let rust_notes: Vec<&bms_model::Note> = model.notes.iter().collect();
    let fixture_notes = &fixture.notes;
    let min_len = rust_notes.len().min(fixture_notes.len());

    for i in 0..min_len {
        let rn = rust_notes[i];
        let fn_ = &fixture_notes[i];

        if rn.wav_id as i32 != fn_.wav_id {
            diffs.push(format!(
                "note[{}] wav_id: rust={} java={}",
                i, rn.wav_id, fn_.wav_id
            ));
        }

        // LN end_wav_id comparison
        if let Some(end_wav_id) = fn_.end_wav_id
            && rn.is_long_note()
            && end_wav_id >= 0
            && rn.end_wav_id as i32 != end_wav_id
        {
            diffs.push(format!(
                "note[{}] end_wav_id: rust={} java={}",
                i, rn.end_wav_id, end_wav_id
            ));
        }
    }

    diffs
}

/// Assert that a Rust BmsModel matches a Java fixture.
/// Panics with detailed diff if differences are found.
pub fn assert_model_matches_fixture(model: &bms_model::BmsModel, fixture: &Fixture) {
    let diffs = compare_model(model, fixture);
    if !diffs.is_empty() {
        panic!(
            "Golden master mismatch ({} differences):\n{}",
            diffs.len(),
            diffs
                .iter()
                .map(|d| format!("  - {}", d))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Assert that a bmson-decoded Rust BmsModel matches a Java fixture.
/// Includes wav_id comparison since bmson uses the same semantics.
pub fn assert_bmson_model_matches_fixture(model: &bms_model::BmsModel, fixture: &Fixture) {
    let diffs = compare_model_bmson(model, fixture);
    if !diffs.is_empty() {
        panic!(
            "Golden master mismatch ({} differences):\n{}",
            diffs.len(),
            diffs
                .iter()
                .map(|d| format!("  - {}", d))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

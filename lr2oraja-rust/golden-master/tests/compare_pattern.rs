// Golden master tests for Phase 3: Pattern Shuffle
//
// Compares Rust lane shuffle implementations against Java fixture output.

use std::path::Path;

use golden_master::pattern_fixtures::{LaneShuffleFixture, PlayableRandomFixture};

use bms_pattern::lane_shuffle::{
    LaneCrossShuffle, LaneMirrorShuffle, LanePlayableRandomShuffle, LaneRandomShuffle,
    LaneRotateShuffle, PlayerFlipShuffle,
};
use bms_pattern::modifier::get_keys;

fn fixture_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .leak()
}

// =========================================================================
// Lane Shuffle Mapping Tests
// =========================================================================

#[test]
fn golden_master_lane_shuffle_mappings() {
    let fixture_path = fixture_dir().join("pattern_lane_shuffle.json");
    if !fixture_path.exists() {
        eprintln!(
            "Fixture not found: {}. Run `just golden-master-pattern-gen` first.",
            fixture_path.display()
        );
        return;
    }

    let content = std::fs::read_to_string(&fixture_path).expect("Failed to read fixture");
    let fixture: LaneShuffleFixture =
        serde_json::from_str(&content).expect("Failed to parse fixture");

    let mut pass = 0;
    let mut fail = 0;

    for (i, tc) in fixture.test_cases.iter().enumerate() {
        let mode = golden_master::mode_hint_to_play_mode(&tc.mode)
            .unwrap_or_else(|| panic!("Unknown mode: {}", tc.mode));

        let rust_mapping = match tc.modifier_type.as_str() {
            "MIRROR" => {
                let shuffle = LaneMirrorShuffle::new(tc.player, tc.contains_scratch);
                let keys = get_keys(mode, tc.player, tc.contains_scratch);
                shuffle.make_random(&keys, tc.key_count)
            }
            "ROTATE" => {
                let seed = tc.seed.expect("Rotate requires seed");
                let shuffle = LaneRotateShuffle::new(tc.player, tc.contains_scratch, seed);
                let keys = get_keys(mode, tc.player, tc.contains_scratch);
                shuffle.make_random(&keys, tc.key_count)
            }
            "RANDOM" => {
                let seed = tc.seed.expect("Random requires seed");
                let shuffle = LaneRandomShuffle::new(tc.player, tc.contains_scratch, seed);
                let keys = get_keys(mode, tc.player, tc.contains_scratch);
                shuffle.make_random(&keys, tc.key_count)
            }
            "CROSS" => {
                let shuffle = LaneCrossShuffle::new(tc.player, tc.contains_scratch);
                let keys = get_keys(mode, tc.player, tc.contains_scratch);
                shuffle.make_random(&keys, tc.key_count)
            }
            "FLIP" => {
                let shuffle = PlayerFlipShuffle::new();
                shuffle.make_random(tc.key_count, mode.player_count())
            }
            other => panic!("Unknown modifier type: {other}"),
        };

        if rust_mapping == tc.mapping {
            pass += 1;
        } else {
            fail += 1;
            eprintln!(
                "FAIL case[{i}] {modifier} mode={mode} seed={seed:?} scratch={scratch} player={player}",
                modifier = tc.modifier_type,
                mode = tc.mode,
                seed = tc.seed,
                scratch = tc.contains_scratch,
                player = tc.player,
            );
            eprintln!("  expected: {:?}", tc.mapping);
            eprintln!("  actual:   {:?}", rust_mapping);
        }
    }

    println!(
        "\nLane shuffle mapping results: {pass} passed, {fail} failed (total {})",
        fixture.test_cases.len()
    );
    assert_eq!(fail, 0, "{fail} lane shuffle mapping test(s) failed");
}

// =========================================================================
// Playable Random Tests
// =========================================================================

#[test]
fn golden_master_playable_random() {
    let fixture_path = fixture_dir().join("pattern_playable_random.json");
    if !fixture_path.exists() {
        eprintln!(
            "Fixture not found: {}. Run `just golden-master-pattern-gen` first.",
            fixture_path.display()
        );
        return;
    }

    let content = std::fs::read_to_string(&fixture_path).expect("Failed to read fixture");
    let fixture: PlayableRandomFixture =
        serde_json::from_str(&content).expect("Failed to parse fixture");

    let mut pass = 0;
    let mut fail = 0;

    for (i, tc) in fixture.test_cases.iter().enumerate() {
        let mode = golden_master::mode_hint_to_play_mode(&tc.mode)
            .unwrap_or_else(|| panic!("Unknown mode: {}", tc.mode));

        // Build a minimal BmsModel with the chord patterns encoded as notes
        let model = build_model_from_chord_patterns(mode, &tc.chord_patterns);

        let keys = get_keys(mode, 0, false);
        let shuffle = LanePlayableRandomShuffle::new(0, false, tc.seed);
        let rust_mapping = shuffle.make_random(&keys, &model);

        // Verify candidate count by running the search directly
        let candidates =
            bms_pattern::lane_shuffle::search_no_murioshi_combinations(&tc.chord_patterns);
        let rust_candidate_count = candidates.len();

        let mapping_ok = rust_mapping == tc.mapping;
        let count_ok = rust_candidate_count == tc.candidate_count;

        if mapping_ok && count_ok {
            pass += 1;
        } else {
            fail += 1;
            eprintln!(
                "FAIL case[{i}] seed={seed} is_fallback={fallback}",
                seed = tc.seed,
                fallback = tc.is_fallback,
            );
            if !mapping_ok {
                eprintln!("  mapping expected: {:?}", tc.mapping);
                eprintln!("  mapping actual:   {:?}", rust_mapping);
            }
            if !count_ok {
                eprintln!(
                    "  candidate_count expected: {}, actual: {}",
                    tc.candidate_count, rust_candidate_count
                );
            }
        }
    }

    println!(
        "\nPlayable random results: {pass} passed, {fail} failed (total {})",
        fixture.test_cases.len()
    );
    assert_eq!(fail, 0, "{fail} playable random test(s) failed");
}

/// Build a minimal BmsModel with notes that produce the given chord patterns.
///
/// Each chord pattern is a bitmask where bit `j` means lane `j` has a note.
/// We place one timeline per pattern, with Normal notes on the active lanes.
fn build_model_from_chord_patterns(
    mode: bms_model::PlayMode,
    chord_patterns: &[u32],
) -> bms_model::BmsModel {
    let mut notes = Vec::new();
    let base_time = 1_000_000i64; // 1 second

    for (idx, &pattern) in chord_patterns.iter().enumerate() {
        let time_us = base_time + (idx as i64) * 100_000;
        for lane in 0..9 {
            if (pattern >> lane) & 1 == 1 {
                notes.push(bms_model::Note::normal(lane, time_us, 1));
            }
        }
    }

    bms_model::BmsModel {
        title: String::new(),
        subtitle: String::new(),
        artist: String::new(),
        sub_artist: String::new(),
        genre: String::new(),
        banner: String::new(),
        stage_file: String::new(),
        back_bmp: String::new(),
        preview: String::new(),
        play_level: 0,
        judge_rank: 100,
        total: 300.0,
        difficulty: 0,
        mode,
        ln_type: bms_model::LnType::LongNote,
        player: 1,
        initial_bpm: 130.0,
        bpm_changes: Vec::new(),
        stop_events: Vec::new(),
        timelines: Vec::new(),
        notes,
        wav_defs: Default::default(),
        bmp_defs: Default::default(),
        md5: String::new(),
        sha256: String::new(),
        total_measures: 4,
        total_time_us: 0,
    }
}

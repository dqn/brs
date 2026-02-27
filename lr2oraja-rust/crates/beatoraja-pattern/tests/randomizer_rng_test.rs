// Phase 49: Randomizer RNG divergence tests
//
// Demonstrates that StdRng and JavaRandom produce completely different
// sequences from the same seed, and verifies JavaRandom determinism.
//
// Phase 51: RED-ONLY tests exposing RNG correctness issues.
// These tests document vulnerabilities; they do NOT fix the code.

use beatoraja_core::player_config::PlayerConfig;
use beatoraja_pattern::java_random::JavaRandom;
use beatoraja_pattern::long_note_modifier::LongNoteModifier;
use beatoraja_pattern::mine_note_modifier::MineNoteModifier;
use beatoraja_pattern::note_shuffle_modifier::NoteShuffleModifier;
use beatoraja_pattern::pattern_modifier::PatternModifier;
use beatoraja_pattern::random::Random;
use beatoraja_pattern::randomizer::RandomizerBase;
use bms_model::bms_model::BMSModel;
use bms_model::mode::Mode;
use bms_model::note::Note;
use bms_model::time_line::TimeLine;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Phase 49a: StdRng vs JavaRandom diverge from the same seed.
/// This is the core invariant: beatoraja MUST use JavaRandom (LCG),
/// never StdRng, to match Java replay/pattern determinism.
#[test]
fn std_rng_vs_java_random_diverge() {
    let seed: u64 = 42;

    let mut std_rng = StdRng::seed_from_u64(seed);
    let mut java_rng = JavaRandom::new(seed as i64);

    let mut std_results = Vec::with_capacity(100);
    let mut java_results = Vec::with_capacity(100);

    for _ in 0..100 {
        std_results.push(std_rng.gen_range(0..7));
        java_results.push(java_rng.next_int_bounded(7));
    }

    // They MUST differ — if they don't, something is very wrong
    assert_ne!(
        std_results, java_results,
        "StdRng and JavaRandom must produce different sequences from the same seed"
    );

    // Verify they diverge from the very first value
    assert_ne!(
        std_results[0], java_results[0],
        "Even the first generated value should differ between StdRng and JavaRandom"
    );
}

/// Phase 49b: JavaRandom is deterministic — same seed always produces
/// the same sequence. This is the baseline for replay/pattern reproducibility.
#[test]
fn java_random_deterministic_same_seed() {
    let seed: i64 = 42;

    let mut rng1 = JavaRandom::new(seed);
    let mut rng2 = JavaRandom::new(seed);

    let results1: Vec<i32> = (0..100).map(|_| rng1.next_int_bounded(7)).collect();
    let results2: Vec<i32> = (0..100).map(|_| rng2.next_int_bounded(7)).collect();

    assert_eq!(
        results1, results2,
        "JavaRandom with the same seed must produce identical sequences"
    );
}

/// Verify that different seeds produce different sequences (sanity check).
#[test]
fn java_random_different_seeds_diverge() {
    let mut rng1 = JavaRandom::new(42);
    let mut rng2 = JavaRandom::new(99);

    let results1: Vec<i32> = (0..100).map(|_| rng1.next_int_bounded(7)).collect();
    let results2: Vec<i32> = (0..100).map(|_| rng2.next_int_bounded(7)).collect();

    assert_ne!(
        results1, results2,
        "JavaRandom with different seeds should produce different sequences"
    );
}

// ---- Phase 51: RED-ONLY tests for RNG correctness issues ----

/// Helper: create a BMSModel with the given mode and timelines.
fn make_test_model(mode: &Mode, timelines: Vec<TimeLine>) -> BMSModel {
    let mut model = BMSModel::new();
    model.set_all_time_line(timelines);
    model.set_mode(mode.clone());
    model
}

/// 4-2b: SRandomizer determinism baseline — same seed produces identical output.
///
/// Creates an SRandomizer (via NoteShuffleModifier) with the same seed twice,
/// applies it to identical note data, and verifies the outputs match.
/// This is the baseline: seeded StdRng IS deterministic within itself.
/// The problem (tested in randomizer_base_uses_stdrng_not_java_random) is that
/// it's the WRONG RNG algorithm — StdRng instead of JavaRandom.
#[test]
fn s_randomizer_determinism_same_seed() {
    let seed: i64 = 12345;
    let mode = Mode::BEAT_7K;
    let config = PlayerConfig::default();

    // Build two identical models with notes spread across lanes
    let build_model = || {
        let mut timelines = Vec::new();
        for section in 0i32..4 {
            let mut tl = TimeLine::new(section as f64, section as i64 * 1000, 8);
            // Place notes in lanes 0, 2, 4 to give the randomizer something to shuffle
            tl.set_note(0, Some(Note::new_normal(10 + section)));
            tl.set_note(2, Some(Note::new_normal(20 + section)));
            tl.set_note(4, Some(Note::new_normal(30 + section)));
            timelines.push(tl);
        }
        make_test_model(&mode, timelines)
    };

    // First run
    let mut model1 = build_model();
    let mut modifier1 = NoteShuffleModifier::new(Random::SRandom, 0, &mode, &config);
    modifier1.set_seed(seed);
    modifier1.modify(&mut model1);

    // Second run — same seed, same input
    let mut model2 = build_model();
    let mut modifier2 = NoteShuffleModifier::new(Random::SRandom, 0, &mode, &config);
    modifier2.set_seed(seed);
    modifier2.modify(&mut model2);

    // Extract note positions from both runs
    let extract_notes = |model: &BMSModel| -> Vec<Vec<Option<i32>>> {
        model
            .get_all_time_lines()
            .iter()
            .map(|tl| {
                (0..8)
                    .map(|lane| tl.get_note(lane).map(|n| n.get_wav()))
                    .collect()
            })
            .collect()
    };

    let notes1 = extract_notes(&model1);
    let notes2 = extract_notes(&model2);

    // Baseline: same seed → same shuffle. StdRng is deterministic.
    assert_eq!(
        notes1, notes2,
        "SRandomizer with the same seed must produce identical note layouts"
    );
}

/// 4-3a: RandomizerBase stores StdRng, not JavaRandom.
///
/// Seeds RandomizerBase with seed=42 and extracts random values via StdRng.
/// Compares them against JavaRandom(42). They differ, proving the AGENTS.md
/// invariant "JavaRandom LCG in beatoraja-pattern (never StdRng/rand)" is violated.
///
/// Impact: pattern shuffles won't match Java beatoraja's output for the same seed,
/// breaking cross-implementation replay/pattern reproducibility.
#[test]
fn randomizer_base_uses_stdrng_not_java_random() {
    let seed: i64 = 42;

    // RandomizerBase.set_random_seed() creates StdRng::seed_from_u64(seed)
    let mut base = RandomizerBase::new();
    base.set_random_seed(seed);

    // Extract 20 values from the StdRng inside RandomizerBase
    let stdrng_values: Vec<i32> = (0..20).map(|_| base.random.gen_range(0..7)).collect();

    // Now get 20 values from JavaRandom with the same seed
    let mut java_rng = JavaRandom::new(seed);
    let java_values: Vec<i32> = (0..20).map(|_| java_rng.next_int_bounded(7)).collect();

    // These MUST differ — RandomizerBase uses StdRng, not JavaRandom.
    // This is the bug: AGENTS.md says "JavaRandom LCG in beatoraja-pattern
    // (never StdRng/rand)" but RandomizerBase.random is StdRng.
    assert_ne!(
        stdrng_values, java_values,
        "RandomizerBase produces StdRng sequences, not JavaRandom sequences — \
         this violates the JavaRandom LCG invariant from AGENTS.md"
    );

    // Verify the field type is actually StdRng by confirming it matches
    // an independently-seeded StdRng
    let mut expected_stdrng = StdRng::seed_from_u64(seed as u64);
    let expected_values: Vec<i32> = (0..20).map(|_| expected_stdrng.gen_range(0..7)).collect();

    assert_eq!(
        stdrng_values, expected_values,
        "RandomizerBase.random IS StdRng — confirming the wrong RNG is used"
    );
}

/// 4-3b: MineNoteModifier.modify() ignores the seed field.
///
/// In AddRandom mode (mode=1), MineNoteModifier uses `rand::random::<f64>()`
/// (the thread-local, unseeded global RNG) to decide whether to place mine notes.
/// The seed set via set_seed() is stored but never consumed by modify().
///
/// This means: even with the same seed, mine note placement is non-deterministic
/// across runs. Replays and pattern reproducibility are broken for mine notes.
#[test]
fn mine_note_modifier_ignores_seed() {
    let seed: i64 = 42;
    let mode = Mode::BEAT_7K;

    // Build a model with enough blank lanes for AddRandom to act on.
    // AddRandom places mines in blank lanes with probability ~10% (rand::random() > 0.9).
    // We use many timelines to increase the chance of observing non-determinism.
    let build_model = || {
        let mut timelines = Vec::new();
        for section in 0..50 {
            let mut tl = TimeLine::new(section as f64, section * 1000, 8);
            // Place a note only in lane 0; lanes 1-7 are blank and eligible for mines
            tl.set_note(0, Some(Note::new_normal(10)));
            timelines.push(tl);
        }
        make_test_model(&mode, timelines)
    };

    // Run modify 10 times with the same seed, collect results
    let mut results: Vec<Vec<Vec<Option<i32>>>> = Vec::new();
    for _ in 0..10 {
        let mut model = build_model();
        let mut modifier = MineNoteModifier::with_mode(1); // AddRandom
        modifier.set_seed(seed);
        modifier.modify(&mut model);

        let layout: Vec<Vec<Option<i32>>> = model
            .get_all_time_lines()
            .iter()
            .map(|tl| {
                (0..8)
                    .map(|lane| tl.get_note(lane).map(|n| n.get_wav()))
                    .collect()
            })
            .collect();
        results.push(layout);
    }

    // If modify() honored the seed, all 10 results would be identical.
    // Because it uses rand::random() (unseeded global RNG), they likely differ.
    //
    // NOTE: There is a small probability all 10 runs produce the same output
    // (each lane has ~10% chance of a mine, 50 timelines * 7 lanes = 350 decisions).
    // In practice this virtually never happens, but we document it as a known
    // limitation of statistical tests.
    let all_identical = results.windows(2).all(|w| w[0] == w[1]);

    // We expect non-determinism. If all results are identical, the test should
    // be re-examined — but for a RED test, we document the bug either way.
    // The key insight: set_seed(42) stores the seed but modify() never reads it.
    assert!(
        !all_identical,
        "MineNoteModifier.modify() should be non-deterministic because it uses \
         rand::random() instead of the seeded RNG — the seed field is ignored. \
         If this assertion fails, it's a statistical fluke (re-run the test)."
    );
}

/// 4-3b: LongNoteModifier.modify() ignores the seed field.
///
/// In Remove mode (default), LongNoteModifier uses `rand::random::<f64>()`
/// to decide whether to convert long notes to normal notes (probability = rate).
/// With rate=1.0 (default), ALL long notes get removed, making the output
/// deterministic regardless of RNG. So we use rate=0.5 to expose the bug.
///
/// The seed set via set_seed() is stored but never consumed by modify().
/// This means long note removal patterns differ across runs for the same seed.
#[test]
fn long_note_modifier_ignores_seed() {
    let seed: i64 = 42;
    let mode = Mode::BEAT_7K;

    // Build a model with long notes. Use rate=0.5 so ~half are removed randomly.
    // We need start+end pairs: a start note followed by an end note in the next timeline.
    let build_model = || {
        let mut timelines = Vec::new();
        for section in 0..100 {
            let mut tl = TimeLine::new(section as f64, section * 1000, 8);
            if section % 2 == 0 {
                // Start of long note in lanes 0-6
                for lane in 0..7 {
                    let mut ln = Note::new_long(10 + lane);
                    ln.set_long_note_type(1); // TYPE_LONGNOTE
                    tl.set_note(lane, Some(ln));
                }
            } else {
                // End of long note in lanes 0-6
                for lane in 0..7 {
                    let mut end = Note::new_long(-2);
                    end.set_end(true);
                    tl.set_note(lane, Some(end));
                }
            }
            timelines.push(tl);
        }
        make_test_model(&mode, timelines)
    };

    // Run modify 10 times with the same seed and rate=0.5
    let mut results: Vec<Vec<Vec<Option<i32>>>> = Vec::new();
    for _ in 0..10 {
        let mut model = build_model();
        let mut modifier = LongNoteModifier::with_params(0, 0.5); // Remove mode, 50% rate
        modifier.set_seed(seed);
        modifier.modify(&mut model);

        let layout: Vec<Vec<Option<i32>>> = model
            .get_all_time_lines()
            .iter()
            .map(|tl| {
                (0..8)
                    .map(|lane| tl.get_note(lane).map(|n| n.get_wav()))
                    .collect()
            })
            .collect();
        results.push(layout);
    }

    // If modify() honored the seed, all 10 results would be identical.
    // Because it uses rand::random() (unseeded global RNG), they likely differ.
    let all_identical = results.windows(2).all(|w| w[0] == w[1]);

    assert!(
        !all_identical,
        "LongNoteModifier.modify() should be non-deterministic because it uses \
         rand::random() instead of the seeded RNG — the seed field is ignored. \
         If this assertion fails, it's a statistical fluke (re-run the test)."
    );
}

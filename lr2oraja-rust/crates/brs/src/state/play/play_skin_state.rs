// Play-specific skin state synchronization.
//
// Updates SharedGameState integers/booleans with play-specific values
// (gauge, score, combo, BPM, judge counts, etc.) each frame.

use bms_rule::GrooveGauge;
use bms_rule::judge_manager::JudgeManager;
use bms_skin::property_id::{
    NUMBER_BAD, NUMBER_COMBO, NUMBER_EARLY_BAD, NUMBER_EARLY_GOOD, NUMBER_EARLY_GREAT,
    NUMBER_EARLY_MISS, NUMBER_EARLY_PERFECT, NUMBER_EARLY_POOR, NUMBER_GOOD, NUMBER_GREAT,
    NUMBER_GROOVEGAUGE, NUMBER_GROOVEGAUGE_AFTERDOT, NUMBER_LATE_BAD, NUMBER_LATE_GOOD,
    NUMBER_LATE_GREAT, NUMBER_LATE_MISS, NUMBER_LATE_PERFECT, NUMBER_LATE_POOR, NUMBER_MAXCOMBO2,
    NUMBER_MISS, NUMBER_NOWBPM, NUMBER_PERFECT, NUMBER_POOR, NUMBER_SCORE2, NUMBER_TOTALNOTES2,
};

use crate::game_state::SharedGameState;

/// Synchronize play-specific state into SharedGameState for skin rendering.
///
/// Called once per frame during the Playing phase.
#[allow(dead_code)]
pub fn sync_play_state(
    state: &mut SharedGameState,
    jm: &JudgeManager,
    gauge: &GrooveGauge,
    current_bpm: i32,
) {
    let score = jm.score();

    // Gauge value (integer part and fractional part)
    let gauge_val = gauge.value();
    state.integers.insert(NUMBER_GROOVEGAUGE, gauge_val as i32);
    state.integers.insert(
        NUMBER_GROOVEGAUGE_AFTERDOT,
        ((gauge_val % 1.0) * 100.0) as i32,
    );

    // Score
    state.integers.insert(NUMBER_SCORE2, score.exscore());

    // Combo
    state.integers.insert(NUMBER_COMBO, jm.combo());
    state.integers.insert(NUMBER_MAXCOMBO2, jm.max_combo());

    // Total notes
    state.integers.insert(NUMBER_TOTALNOTES2, score.notes);

    // BPM
    state.integers.insert(NUMBER_NOWBPM, current_bpm);

    // Judge counts (total)
    state
        .integers
        .insert(NUMBER_PERFECT, score.judge_count(bms_rule::JUDGE_PG));
    state
        .integers
        .insert(NUMBER_GREAT, score.judge_count(bms_rule::JUDGE_GR));
    state
        .integers
        .insert(NUMBER_GOOD, score.judge_count(bms_rule::JUDGE_GD));
    state
        .integers
        .insert(NUMBER_BAD, score.judge_count(bms_rule::JUDGE_BD));
    state
        .integers
        .insert(NUMBER_POOR, score.judge_count(bms_rule::JUDGE_PR));
    state
        .integers
        .insert(NUMBER_MISS, score.judge_count(bms_rule::JUDGE_MS));

    // Judge counts (early/late)
    state.integers.insert(
        NUMBER_EARLY_PERFECT,
        score.judge_count_early(bms_rule::JUDGE_PG),
    );
    state.integers.insert(
        NUMBER_LATE_PERFECT,
        score.judge_count_late(bms_rule::JUDGE_PG),
    );
    state.integers.insert(
        NUMBER_EARLY_GREAT,
        score.judge_count_early(bms_rule::JUDGE_GR),
    );
    state.integers.insert(
        NUMBER_LATE_GREAT,
        score.judge_count_late(bms_rule::JUDGE_GR),
    );
    state.integers.insert(
        NUMBER_EARLY_GOOD,
        score.judge_count_early(bms_rule::JUDGE_GD),
    );
    state
        .integers
        .insert(NUMBER_LATE_GOOD, score.judge_count_late(bms_rule::JUDGE_GD));
    state.integers.insert(
        NUMBER_EARLY_BAD,
        score.judge_count_early(bms_rule::JUDGE_BD),
    );
    state
        .integers
        .insert(NUMBER_LATE_BAD, score.judge_count_late(bms_rule::JUDGE_BD));
    state.integers.insert(
        NUMBER_EARLY_POOR,
        score.judge_count_early(bms_rule::JUDGE_PR),
    );
    state
        .integers
        .insert(NUMBER_LATE_POOR, score.judge_count_late(bms_rule::JUDGE_PR));
    state.integers.insert(
        NUMBER_EARLY_MISS,
        score.judge_count_early(bms_rule::JUDGE_MS),
    );
    state
        .integers
        .insert(NUMBER_LATE_MISS, score.judge_count_late(bms_rule::JUDGE_MS));

    // Gauge type float (for skin gauge rendering)
    state
        .floats
        .insert(bms_skin::property_id::FLOAT_GROOVEGAUGE_1P, gauge_val);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::{LaneProperty, PlayMode};
    use bms_rule::gauge_property::GaugeType;
    use bms_rule::judge_manager::JudgeConfig;
    use bms_rule::{JudgeAlgorithm, PlayerRule};

    fn make_judge_manager(autoplay: bool) -> (JudgeManager, Vec<bms_model::Note>, GrooveGauge) {
        // Create a minimal set of notes for testing
        let notes = vec![bms_model::Note {
            lane: 1,
            note_type: bms_model::NoteType::Normal,
            time_us: 1_000_000,
            end_time_us: 0,
            end_wav_id: 0,
            wav_id: 1,
            damage: 0,
            pair_index: usize::MAX,
            micro_starttime: 0,
            micro_duration: 0,
        }];

        let rule = PlayerRule::lr2();
        let gauge = GrooveGauge::new(&rule.gauge, GaugeType::Normal, 300.0, 1);

        let lp = LaneProperty::new(PlayMode::Beat7K);
        let config = JudgeConfig {
            notes: &notes,
            play_mode: PlayMode::Beat7K,
            ln_type: bms_model::LnType::LongNote,
            judge_rank: 100,
            judge_window_rate: [100, 100, 100],
            scratch_judge_window_rate: [100, 100, 100],
            algorithm: JudgeAlgorithm::Combo,
            autoplay,
            judge_property: &rule.judge,
            lane_property: Some(&lp),
        };
        let jm = JudgeManager::new(&config);
        (jm, notes, gauge)
    }

    #[test]
    fn sync_populates_gauge_value() {
        let (jm, _notes, gauge) = make_judge_manager(true);
        let mut state = SharedGameState::default();

        sync_play_state(&mut state, &jm, &gauge, 150);

        // Normal gauge starts at 20
        assert_eq!(*state.integers.get(&NUMBER_GROOVEGAUGE).unwrap(), 20);
    }

    #[test]
    fn sync_populates_bpm() {
        let (jm, _notes, gauge) = make_judge_manager(true);
        let mut state = SharedGameState::default();

        sync_play_state(&mut state, &jm, &gauge, 175);

        assert_eq!(*state.integers.get(&NUMBER_NOWBPM).unwrap(), 175);
    }

    #[test]
    fn sync_populates_judge_counts() {
        let (jm, _notes, gauge) = make_judge_manager(true);
        let mut state = SharedGameState::default();

        sync_play_state(&mut state, &jm, &gauge, 150);

        // Initially all zeros
        assert_eq!(*state.integers.get(&NUMBER_PERFECT).unwrap(), 0);
        assert_eq!(*state.integers.get(&NUMBER_GREAT).unwrap(), 0);
    }

    #[test]
    fn sync_populates_score() {
        let (jm, _notes, gauge) = make_judge_manager(true);
        let mut state = SharedGameState::default();

        sync_play_state(&mut state, &jm, &gauge, 150);

        assert_eq!(*state.integers.get(&NUMBER_SCORE2).unwrap(), 0);
        assert_eq!(*state.integers.get(&NUMBER_COMBO).unwrap(), 0);
    }

    #[test]
    fn sync_populates_gauge_float() {
        let (jm, _notes, gauge) = make_judge_manager(true);
        let mut state = SharedGameState::default();

        sync_play_state(&mut state, &jm, &gauge, 150);

        let val = *state
            .floats
            .get(&bms_skin::property_id::FLOAT_GROOVEGAUGE_1P)
            .unwrap();
        assert!((val - 20.0).abs() < 1e-6);
    }
}

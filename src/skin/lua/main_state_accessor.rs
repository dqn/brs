use crate::skin::renderer::SkinStateSnapshot;
use crate::skin::skin_property::TIMER_OFF_VALUE;
use crate::state::play::play_state::PlayState;

/// Bridge between PlayState and SkinStateSnapshot.
/// Extracts all needed values from PlayState for the skin renderer.
pub struct MainStateAccessor;

impl MainStateAccessor {
    /// Build a SkinStateSnapshot from the current PlayState.
    pub fn snapshot(play_state: &PlayState, time_ms: i64) -> SkinStateSnapshot {
        let mut snap = SkinStateSnapshot {
            time_ms,
            ..Default::default()
        };

        // Populate timers from TimerManager
        Self::populate_timers(&mut snap, play_state);

        // Populate numbers
        Self::populate_numbers(&mut snap, play_state);

        // Populate floats
        Self::populate_floats(&mut snap, play_state);

        snap
    }

    fn populate_timers(snap: &mut SkinStateSnapshot, play_state: &PlayState) {
        use crate::skin::skin_property as sp;
        use crate::state::play::timer_manager::*;

        let tm = play_state.timer();

        // Map internal timer IDs to beatoraja skin property timer IDs
        let timer_map: &[(usize, i32)] = &[
            (TIMER_PLAY, sp::TIMER_PLAY),
            (TIMER_READY, sp::TIMER_READY),
            (TIMER_FAILED, sp::TIMER_FAILED),
            (TIMER_FADEOUT, sp::TIMER_FADEOUT),
            (TIMER_ENDOFNOTE_1P, sp::TIMER_ENDOFNOTE_1P),
            (TIMER_ENDOFNOTE_2P, sp::TIMER_ENDOFNOTE_2P),
        ];

        for &(internal_id, skin_id) in timer_map {
            let value = tm.get(internal_id).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }

        // Bomb timers (1P): lanes 0-7
        for lane in 0..TIMER_BOMB_1P_COUNT {
            let internal = TIMER_BOMB_1P_BASE + lane;
            let skin_id = sp::TIMER_BOMB_1P_SCRATCH + lane as i32;
            let value = tm.get(internal).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }

        // Bomb timers (2P): lanes 0-7
        for lane in 0..TIMER_BOMB_2P_COUNT {
            let internal = TIMER_BOMB_2P_BASE + lane;
            let skin_id = sp::TIMER_BOMB_2P_SCRATCH + lane as i32;
            let value = tm.get(internal).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }

        // Key-on timers (1P)
        for lane in 0..TIMER_KEYON_1P_COUNT {
            let internal = TIMER_KEYON_1P_BASE + lane;
            let skin_id = sp::TIMER_KEYON_1P_SCRATCH + lane as i32;
            let value = tm.get(internal).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }

        // Key-on timers (2P)
        for lane in 0..TIMER_KEYON_2P_COUNT {
            let internal = TIMER_KEYON_2P_BASE + lane;
            let skin_id = sp::TIMER_KEYON_2P_SCRATCH + lane as i32;
            let value = tm.get(internal).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }

        // Key-off timers (1P)
        for lane in 0..TIMER_KEYOFF_1P_COUNT {
            let internal = TIMER_KEYOFF_1P_BASE + lane;
            let skin_id = sp::TIMER_KEYOFF_1P_SCRATCH + lane as i32;
            let value = tm.get(internal).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }

        // Key-off timers (2P)
        for lane in 0..TIMER_KEYOFF_2P_COUNT {
            let internal = TIMER_KEYOFF_2P_BASE + lane;
            let skin_id = sp::TIMER_KEYOFF_2P_SCRATCH + lane as i32;
            let value = tm.get(internal).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }

        // Judge timers (1P)
        for lane in 0..TIMER_JUDGE_1P_COUNT {
            let internal = TIMER_JUDGE_1P_BASE + lane;
            let skin_id = sp::TIMER_JUDGE_1P + lane as i32;
            let value = tm.get(internal).unwrap_or(TIMER_OFF_VALUE);
            snap.timers.push((skin_id, value));
        }
    }

    fn populate_numbers(snap: &mut SkinStateSnapshot, play_state: &PlayState) {
        use crate::skin::skin_property as sp;

        let js = play_state.judge_score();

        // Judge counts
        snap.numbers
            .push((sp::NUMBER_PERFECT, js.judge_count(0) as i32));
        snap.numbers
            .push((sp::NUMBER_GREAT, js.judge_count(1) as i32));
        snap.numbers
            .push((sp::NUMBER_GOOD, js.judge_count(2) as i32));
        snap.numbers
            .push((sp::NUMBER_BAD, js.judge_count(3) as i32));
        snap.numbers
            .push((sp::NUMBER_POOR, js.judge_count(4) as i32));
        snap.numbers
            .push((sp::NUMBER_MISS, js.judge_count(5) as i32));

        // Early/late counts
        snap.numbers
            .push((sp::NUMBER_EARLY_PERFECT, js.early_counts[0] as i32));
        snap.numbers
            .push((sp::NUMBER_LATE_PERFECT, js.late_counts[0] as i32));
        snap.numbers
            .push((sp::NUMBER_EARLY_GREAT, js.early_counts[1] as i32));
        snap.numbers
            .push((sp::NUMBER_LATE_GREAT, js.late_counts[1] as i32));
        snap.numbers
            .push((sp::NUMBER_EARLY_GOOD, js.early_counts[2] as i32));
        snap.numbers
            .push((sp::NUMBER_LATE_GOOD, js.late_counts[2] as i32));
        snap.numbers
            .push((sp::NUMBER_EARLY_BAD, js.early_counts[3] as i32));
        snap.numbers
            .push((sp::NUMBER_LATE_BAD, js.late_counts[3] as i32));
        snap.numbers
            .push((sp::NUMBER_EARLY_POOR, js.early_counts[4] as i32));
        snap.numbers
            .push((sp::NUMBER_LATE_POOR, js.late_counts[4] as i32));
        snap.numbers
            .push((sp::NUMBER_EARLY_MISS, js.early_counts[5] as i32));
        snap.numbers
            .push((sp::NUMBER_LATE_MISS, js.late_counts[5] as i32));

        // Combo
        snap.numbers.push((sp::NUMBER_COMBO, js.combo as i32));
        snap.numbers
            .push((sp::NUMBER_MAXCOMBO2, js.max_combo as i32));
        // EX score
        let exscore = js.judge_count(0) as i32 * 2 + js.judge_count(1) as i32;
        snap.numbers.push((sp::NUMBER_SCORE2, exscore));

        // Gauge
        let gauge_value = play_state.gauge().value();
        snap.numbers
            .push((sp::NUMBER_GROOVEGAUGE, gauge_value as i32));
        snap.numbers.push((
            sp::NUMBER_GROOVEGAUGE_AFTERDOT,
            ((gauge_value * 10.0) as i32) % 10,
        ));
    }

    fn populate_floats(snap: &mut SkinStateSnapshot, play_state: &PlayState) {
        use crate::skin::skin_property as sp;

        let gauge_value = play_state.gauge().value();
        snap.floats.push((sp::RATE_SCORE, gauge_value / 100.0));
    }
}

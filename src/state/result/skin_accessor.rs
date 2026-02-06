use crate::play::clear_type::ClearType;
use crate::play::judge::judge_manager::JudgeLevel;
use crate::play::score::ScoreRank;
use crate::skin::renderer::SkinStateSnapshot;
use crate::skin::skin_property::*;
use crate::state::result::result_state::{ResultPhase, ResultState};
use crate::state::song_metadata::SongMetadata;

pub struct ResultSkinAccessor;

impl ResultSkinAccessor {
    pub fn snapshot(
        state: &ResultState,
        elapsed_us: i64,
        metadata: &SongMetadata,
    ) -> SkinStateSnapshot {
        let mut snap = SkinStateSnapshot {
            time_ms: elapsed_us / 1000,
            ..Default::default()
        };

        let pr = state.play_result();
        let s = &pr.score;

        // Timers
        snap.timers.insert(TIMER_STARTINPUT, 0);
        snap.timers.insert(TIMER_RESULTGRAPH_BEGIN, 0);
        if state.phase() == ResultPhase::FadeOut {
            snap.timers.insert(TIMER_FADEOUT, elapsed_us);
        }

        // Judge counts
        snap.numbers.insert(
            NUMBER_PERFECT,
            s.judge_count(JudgeLevel::PerfectGreat) as i32,
        );
        snap.numbers
            .insert(NUMBER_GREAT, s.judge_count(JudgeLevel::Great) as i32);
        snap.numbers
            .insert(NUMBER_GOOD, s.judge_count(JudgeLevel::Good) as i32);
        snap.numbers
            .insert(NUMBER_BAD, s.judge_count(JudgeLevel::Bad) as i32);
        snap.numbers
            .insert(NUMBER_POOR, s.judge_count(JudgeLevel::Poor) as i32);
        snap.numbers
            .insert(NUMBER_MISS, s.judge_count(JudgeLevel::Miss) as i32);

        // Early/late counts
        snap.numbers
            .insert(NUMBER_EARLY_PERFECT, s.early_counts[0] as i32);
        snap.numbers
            .insert(NUMBER_LATE_PERFECT, s.late_counts[0] as i32);
        snap.numbers
            .insert(NUMBER_EARLY_GREAT, s.early_counts[1] as i32);
        snap.numbers
            .insert(NUMBER_LATE_GREAT, s.late_counts[1] as i32);
        snap.numbers
            .insert(NUMBER_EARLY_GOOD, s.early_counts[2] as i32);
        snap.numbers
            .insert(NUMBER_LATE_GOOD, s.late_counts[2] as i32);
        snap.numbers
            .insert(NUMBER_EARLY_BAD, s.early_counts[3] as i32);
        snap.numbers
            .insert(NUMBER_LATE_BAD, s.late_counts[3] as i32);
        snap.numbers
            .insert(NUMBER_EARLY_POOR, s.early_counts[4] as i32);
        snap.numbers
            .insert(NUMBER_LATE_POOR, s.late_counts[4] as i32);
        snap.numbers
            .insert(NUMBER_EARLY_MISS, s.early_counts[5] as i32);
        snap.numbers
            .insert(NUMBER_LATE_MISS, s.late_counts[5] as i32);

        // Score
        let exscore = s.exscore();
        snap.numbers.insert(NUMBER_SCORE2, exscore as i32);
        snap.numbers.insert(NUMBER_MAXCOMBO, s.max_combo as i32);
        snap.numbers.insert(NUMBER_MAXCOMBO2, s.max_combo as i32);
        snap.numbers.insert(NUMBER_TOTALNOTES, s.total_notes as i32);
        snap.numbers
            .insert(NUMBER_TOTALNOTES2, s.total_notes as i32);
        snap.numbers
            .insert(NUMBER_GROOVEGAUGE, pr.gauge_value as i32);
        snap.numbers.insert(
            NUMBER_GROOVEGAUGE_AFTERDOT,
            ((pr.gauge_value * 10.0) as i32) % 10,
        );
        snap.numbers.insert(NUMBER_MISSCOUNT, s.min_bp as i32);

        // Score rate
        let rate = s.rate();
        snap.numbers
            .insert(NUMBER_SCORE_RATE, (rate * 100.0) as i32);
        snap.numbers
            .insert(NUMBER_SCORE_RATE_AFTERDOT, ((rate * 1000.0) as i32) % 10);

        // Combo break
        let combo_break = s.judge_count(JudgeLevel::Bad)
            + s.judge_count(JudgeLevel::Poor)
            + s.judge_count(JudgeLevel::Miss);
        snap.numbers.insert(NUMBER_COMBOBREAK, combo_break as i32);

        // Floats
        snap.floats.insert(RATE_SCORE_FINAL, rate);
        snap.floats.insert(RATE_SCORE, pr.gauge_value / 100.0);

        // Judge rate bargraphs
        let total = s.total_notes as f32;
        if total > 0.0 {
            snap.floats.insert(
                BARGRAPH_RATE_PGREAT,
                s.judge_count(JudgeLevel::PerfectGreat) as f32 / total,
            );
            snap.floats.insert(
                BARGRAPH_RATE_GREAT,
                s.judge_count(JudgeLevel::Great) as f32 / total,
            );
            snap.floats.insert(
                BARGRAPH_RATE_GOOD,
                s.judge_count(JudgeLevel::Good) as f32 / total,
            );
            snap.floats.insert(
                BARGRAPH_RATE_BAD,
                s.judge_count(JudgeLevel::Bad) as f32 / total,
            );
            snap.floats.insert(
                BARGRAPH_RATE_POOR,
                (s.judge_count(JudgeLevel::Poor) + s.judge_count(JudgeLevel::Miss)) as f32 / total,
            );
            snap.floats
                .insert(BARGRAPH_RATE_MAXCOMBO, s.max_combo as f32 / total);
            snap.floats
                .insert(BARGRAPH_RATE_EXSCORE, exscore as f32 / (total * 2.0));
        }

        // Clear/Fail options
        snap.options
            .insert(OPTION_RESULT_CLEAR, pr.clear_type != ClearType::Failed);
        snap.options
            .insert(OPTION_RESULT_FAIL, pr.clear_type == ClearType::Failed);

        // Rank options
        snap.options
            .insert(OPTION_1P_AAA, pr.rank >= ScoreRank::AAA);
        snap.options.insert(OPTION_1P_AA, pr.rank == ScoreRank::AA);
        snap.options.insert(OPTION_1P_A, pr.rank == ScoreRank::A);
        snap.options.insert(OPTION_1P_B, pr.rank == ScoreRank::B);
        snap.options.insert(OPTION_1P_C, pr.rank == ScoreRank::C);
        snap.options.insert(OPTION_1P_D, pr.rank == ScoreRank::D);
        snap.options.insert(OPTION_1P_E, pr.rank == ScoreRank::E);
        snap.options.insert(OPTION_1P_F, pr.rank == ScoreRank::F);

        // Song metadata
        snap.strings.insert(STRING_TITLE, metadata.title.clone());
        snap.strings
            .insert(STRING_SUBTITLE, metadata.subtitle.clone());
        snap.strings.insert(STRING_ARTIST, metadata.artist.clone());
        snap.strings
            .insert(STRING_SUBARTIST, metadata.subartist.clone());
        snap.strings.insert(STRING_GENRE, metadata.genre.clone());
        snap.numbers.insert(NUMBER_PLAYLEVEL, metadata.level);
        snap.numbers.insert(NUMBER_MAXBPM, metadata.max_bpm);
        snap.numbers.insert(NUMBER_MINBPM, metadata.min_bpm);

        snap
    }
}

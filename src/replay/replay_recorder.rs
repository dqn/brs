//! Replay recorder for capturing gameplay inputs.

use super::replay_data::{ReplayData, ReplayScore};
use crate::database::ClearType;
use crate::input::KeyInputLog;
use crate::model::{ChartFormat, JudgeRankType, LongNoteMode, PlayMode};
use crate::state::play::PlayResult;

/// Records inputs during gameplay for replay.
pub struct ReplayRecorder {
    replay_data: ReplayData,
}

impl ReplayRecorder {
    /// Create a new recorder for the given chart.
    pub fn new(sha256: String, player_name: String, gauge_type: i32, hi_speed: f32) -> Self {
        Self {
            replay_data: ReplayData::new(sha256, player_name, gauge_type, hi_speed),
        }
    }

    /// Set the input logs from InputLogger.
    pub fn set_input_logs(&mut self, logs: Vec<KeyInputLog>) {
        self.replay_data.input_logs = logs;
    }

    /// Set the final score data.
    pub fn set_score(&mut self, result: &PlayResult, clear_type: ClearType) {
        self.replay_data.metadata.play_mode = match result.play_mode {
            PlayMode::Beat5K => 5,
            PlayMode::Beat7K => 7,
            PlayMode::Beat10K => 10,
            PlayMode::Beat14K => 14,
            PlayMode::PopN5K => 25,
            PlayMode::PopN9K => 29,
        };
        self.replay_data.metadata.long_note_mode = match result.long_note_mode {
            LongNoteMode::Ln => 1,
            LongNoteMode::Cn => 2,
            LongNoteMode::Hcn => 3,
        };
        self.replay_data.metadata.judge_rank = result.judge_rank;
        self.replay_data.metadata.judge_rank_type = match result.judge_rank_type {
            JudgeRankType::BmsRank => 0,
            JudgeRankType::BmsDefExRank => 1,
            JudgeRankType::BmsonJudgeRank => 2,
        };
        self.replay_data.metadata.total = result.total;
        self.replay_data.metadata.source_format = match result.source_format {
            ChartFormat::Bms => 0,
            ChartFormat::Bmson => 1,
        };
        self.replay_data.score = ReplayScore {
            ex_score: result.ex_score(),
            max_combo: result.max_combo(),
            pg_count: result.score.pg_count,
            gr_count: result.score.gr_count,
            gd_count: result.score.gd_count,
            bd_count: result.score.bd_count,
            pr_count: result.score.pr_count,
            ms_count: result.score.ms_count,
            clear_type: clear_type.as_i32(),
            fast_count: result.fast_count,
            slow_count: result.slow_count,
        };
    }

    /// Take the completed replay data.
    pub fn into_replay_data(self) -> ReplayData {
        self.replay_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ChartFormat, JudgeRankType, LongNoteMode, PlayMode};
    use crate::state::play::{GaugeType, Score};

    #[test]
    fn test_replay_recorder_new() {
        let recorder = ReplayRecorder::new("sha256".to_string(), "player".to_string(), 0, 1.0);
        let data = recorder.into_replay_data();
        assert_eq!(data.metadata.sha256, "sha256");
        assert_eq!(data.metadata.player_name, "player");
    }

    #[test]
    fn test_replay_recorder_set_input_logs() {
        let mut recorder = ReplayRecorder::new("sha256".to_string(), "player".to_string(), 0, 1.0);

        let logs = vec![
            KeyInputLog {
                time_us: 1000,
                lane: 0,
                pressed: true,
            },
            KeyInputLog {
                time_us: 2000,
                lane: 0,
                pressed: false,
            },
        ];
        recorder.set_input_logs(logs);

        let data = recorder.into_replay_data();
        assert_eq!(data.input_logs.len(), 2);
    }

    #[test]
    fn test_replay_recorder_set_score() {
        let mut recorder = ReplayRecorder::new("sha256".to_string(), "player".to_string(), 0, 1.0);

        let mut score = Score::new(150);
        score.pg_count = 100;
        score.gr_count = 50;
        score.max_combo = 150;

        let result = PlayResult::new(
            score,
            100.0,
            GaugeType::Normal,
            true,
            60000.0,
            10,
            5,
            PlayMode::Beat7K,
            LongNoteMode::Ln,
            2,
            JudgeRankType::BmsRank,
            200.0,
            ChartFormat::Bms,
        );

        recorder.set_score(&result, ClearType::Normal);

        let data = recorder.into_replay_data();
        assert_eq!(data.score.ex_score, 250); // 100*2 + 50
        assert_eq!(data.score.max_combo, 150);
        assert_eq!(data.score.pg_count, 100);
        assert_eq!(data.score.gr_count, 50);
        assert_eq!(data.score.clear_type, ClearType::Normal.as_i32());
    }
}

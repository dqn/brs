//! Replay player for playback of recorded inputs.

use crate::input::KeyInputLog;
use crate::model::note::{LANE_COUNT, Lane};

/// Plays back recorded inputs during replay.
pub struct ReplayPlayer {
    logs: Vec<KeyInputLog>,
    current_index: usize,
    /// Simulated key states for 8 lanes.
    lane_states: [bool; LANE_COUNT],
    /// Just pressed this frame.
    just_pressed: [bool; LANE_COUNT],
    /// Just released this frame.
    just_released: [bool; LANE_COUNT],
    /// Press timestamps in microseconds.
    press_times: [u64; LANE_COUNT],
    /// Release timestamps in microseconds.
    release_times: [u64; LANE_COUNT],
}

impl ReplayPlayer {
    /// Create a new replay player with the given input logs.
    pub fn new(logs: Vec<KeyInputLog>) -> Self {
        Self {
            logs,
            current_index: 0,
            lane_states: [false; LANE_COUNT],
            just_pressed: [false; LANE_COUNT],
            just_released: [false; LANE_COUNT],
            press_times: [0; LANE_COUNT],
            release_times: [0; LANE_COUNT],
        }
    }

    /// Update the player state for the given time.
    /// Processes all events up to current_time_us.
    pub fn update(&mut self, current_time_us: u64) {
        // Reset frame state.
        self.just_pressed = [false; LANE_COUNT];
        self.just_released = [false; LANE_COUNT];

        // Process all events up to current time.
        while self.current_index < self.logs.len() {
            let log = &self.logs[self.current_index];
            if log.time_us > current_time_us {
                break;
            }

            let lane = log.lane as usize;
            if lane < LANE_COUNT {
                if log.pressed {
                    if !self.lane_states[lane] {
                        self.just_pressed[lane] = true;
                        self.press_times[lane] = log.time_us;
                    }
                    self.lane_states[lane] = true;
                } else {
                    if self.lane_states[lane] {
                        self.just_released[lane] = true;
                        self.release_times[lane] = log.time_us;
                    }
                    self.lane_states[lane] = false;
                }
            }

            self.current_index += 1;
        }
    }

    /// Check if a lane is currently pressed.
    pub fn is_pressed(&self, lane: Lane) -> bool {
        self.lane_states[lane.index()]
    }

    /// Check if a lane was just pressed this frame.
    pub fn just_pressed(&self, lane: Lane) -> bool {
        self.just_pressed[lane.index()]
    }

    /// Check if a lane was just released this frame.
    pub fn just_released(&self, lane: Lane) -> bool {
        self.just_released[lane.index()]
    }

    /// Get the press timestamp for a lane in microseconds.
    pub fn press_time_us(&self, lane: Lane) -> u64 {
        self.press_times[lane.index()]
    }

    /// Get the release timestamp for a lane in microseconds.
    pub fn release_time_us(&self, lane: Lane) -> u64 {
        self.release_times[lane.index()]
    }

    /// Check if all events have been processed.
    pub fn is_finished(&self) -> bool {
        self.current_index >= self.logs.len()
    }

    /// Reset the player to the beginning.
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.lane_states = [false; LANE_COUNT];
        self.just_pressed = [false; LANE_COUNT];
        self.just_released = [false; LANE_COUNT];
        self.press_times = [0; LANE_COUNT];
        self.release_times = [0; LANE_COUNT];
    }

    /// Seek to a specific time position.
    /// 指定した時刻にシークする。
    pub fn seek(&mut self, time_us: u64) {
        self.reset();
        self.update(time_us);
        self.just_pressed = [false; LANE_COUNT];
        self.just_released = [false; LANE_COUNT];
    }

    /// Get the number of input logs.
    pub fn log_count(&self) -> usize {
        self.logs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_player_new() {
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
        let player = ReplayPlayer::new(logs);
        assert_eq!(player.log_count(), 2);
        assert!(!player.is_finished());
    }

    #[test]
    fn test_replay_player_update() {
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
        let mut player = ReplayPlayer::new(logs);

        // Before any events
        player.update(500);
        assert!(!player.is_pressed(Lane::Scratch));
        assert!(!player.just_pressed(Lane::Scratch));

        // After first event
        player.update(1500);
        assert!(player.is_pressed(Lane::Scratch));
        assert!(player.just_pressed(Lane::Scratch));
        assert_eq!(player.press_time_us(Lane::Scratch), 1000);

        // After just_pressed should be reset
        player.update(1600);
        assert!(player.is_pressed(Lane::Scratch));
        assert!(!player.just_pressed(Lane::Scratch));

        // After release
        player.update(2500);
        assert!(!player.is_pressed(Lane::Scratch));
        assert!(player.just_released(Lane::Scratch));
        assert_eq!(player.release_time_us(Lane::Scratch), 2000);

        assert!(player.is_finished());
    }

    #[test]
    fn test_replay_player_reset() {
        let logs = vec![KeyInputLog {
            time_us: 1000,
            lane: 1,
            pressed: true,
        }];
        let mut player = ReplayPlayer::new(logs);

        player.update(2000);
        assert!(player.is_pressed(Lane::Key1));
        assert!(player.is_finished());

        player.reset();
        assert!(!player.is_pressed(Lane::Key1));
        assert!(!player.is_finished());
    }
}

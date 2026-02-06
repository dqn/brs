use anyhow::Result;

use crate::state::game_state::{GameState, StateTransition};

/// Phase of the decide screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecidePhase {
    /// Loading resources (BMS, audio, skin).
    Loading,
    /// Displaying the decide screen.
    Display,
    /// Fading out to transition.
    FadeOut,
}

/// Configuration for the decide screen.
pub struct DecideConfig {
    /// Path to the BMS file to load.
    pub bms_path: String,
    /// SHA-256 hash of the chart.
    pub sha256: String,
    /// Display duration in microseconds before auto-advance.
    pub scene_duration_us: i64,
    /// Fade-out duration in microseconds.
    pub fadeout_duration_us: i64,
}

impl Default for DecideConfig {
    fn default() -> Self {
        Self {
            bms_path: String::new(),
            sha256: String::new(),
            scene_duration_us: 3_000_000, // 3 seconds
            fadeout_duration_us: 500_000, // 0.5 seconds
        }
    }
}

/// State for the decide screen (loading/preview before play).
///
/// Corresponds to beatoraja's MusicDecide. Displays song info while
/// resources are being loaded, then transitions to play.
pub struct DecideState {
    phase: DecidePhase,
    config: DecideConfig,
    /// Whether the user cancelled (returns to select instead of play).
    cancelled: bool,
    /// Elapsed time in microseconds since entering the state.
    elapsed_us: i64,
    /// Time at which fade-out started (None if not fading).
    fadeout_start_us: Option<i64>,
    /// Whether resource loading is complete.
    loading_complete: bool,
}

impl DecideState {
    pub fn new(config: DecideConfig) -> Self {
        Self {
            phase: DecidePhase::Loading,
            config,
            cancelled: false,
            elapsed_us: 0,
            fadeout_start_us: None,
            loading_complete: false,
        }
    }

    /// Get the current phase.
    pub fn phase(&self) -> DecidePhase {
        self.phase
    }

    /// Get the configuration.
    pub fn config(&self) -> &DecideConfig {
        &self.config
    }

    /// Whether the user cancelled (should return to select).
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Signal that resource loading is complete.
    pub fn set_loading_complete(&mut self) {
        self.loading_complete = true;
    }

    /// Request cancellation (return to select).
    pub fn cancel(&mut self) {
        if self.phase != DecidePhase::FadeOut {
            self.cancelled = true;
            self.start_fadeout();
        }
    }

    /// Request advance to play (skip remaining display time).
    pub fn advance(&mut self) {
        if self.phase == DecidePhase::Display {
            self.start_fadeout();
        }
    }

    fn start_fadeout(&mut self) {
        if self.phase != DecidePhase::FadeOut {
            self.phase = DecidePhase::FadeOut;
            self.fadeout_start_us = Some(self.elapsed_us);
        }
    }
}

impl GameState for DecideState {
    fn create(&mut self) -> Result<()> {
        self.phase = DecidePhase::Loading;
        self.elapsed_us = 0;
        self.cancelled = false;
        self.fadeout_start_us = None;
        Ok(())
    }

    fn update(&mut self, dt_us: i64) -> Result<StateTransition> {
        self.elapsed_us += dt_us;

        match self.phase {
            DecidePhase::Loading => {
                if self.loading_complete {
                    self.phase = DecidePhase::Display;
                }
                Ok(StateTransition::None)
            }
            DecidePhase::Display => {
                // Auto-advance after scene duration
                if self.elapsed_us >= self.config.scene_duration_us {
                    self.start_fadeout();
                }
                Ok(StateTransition::None)
            }
            DecidePhase::FadeOut => {
                let fadeout_elapsed =
                    self.elapsed_us - self.fadeout_start_us.unwrap_or(self.elapsed_us);
                if fadeout_elapsed >= self.config.fadeout_duration_us {
                    if self.cancelled {
                        Ok(StateTransition::Back)
                    } else {
                        Ok(StateTransition::Next)
                    }
                } else {
                    Ok(StateTransition::None)
                }
            }
        }
    }

    fn dispose(&mut self) {
        self.phase = DecidePhase::Loading;
        self.fadeout_start_us = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> DecideConfig {
        DecideConfig {
            bms_path: "test.bms".to_string(),
            sha256: "abc123".to_string(),
            scene_duration_us: 2_000_000,
            fadeout_duration_us: 500_000,
        }
    }

    #[test]
    fn initial_phase_is_loading() {
        let state = DecideState::new(make_config());
        assert_eq!(state.phase(), DecidePhase::Loading);
    }

    #[test]
    fn loading_to_display_on_complete() {
        let mut state = DecideState::new(make_config());
        state.create().unwrap();
        assert_eq!(state.phase(), DecidePhase::Loading);

        state.set_loading_complete();
        state.update(16_667).unwrap();
        assert_eq!(state.phase(), DecidePhase::Display);
    }

    #[test]
    fn display_auto_advances_after_scene_duration() {
        let mut state = DecideState::new(make_config());
        state.create().unwrap();
        state.set_loading_complete();
        state.update(16_667).unwrap(); // -> Display

        // Before scene duration
        state.update(1_000_000).unwrap();
        assert_eq!(state.phase(), DecidePhase::Display);

        // After scene duration (total > 2_000_000)
        state.update(1_000_000).unwrap();
        assert_eq!(state.phase(), DecidePhase::FadeOut);
    }

    #[test]
    fn fadeout_transitions_to_next() {
        let mut state = DecideState::new(make_config());
        state.create().unwrap();
        state.set_loading_complete();
        state.update(16_667).unwrap(); // -> Display

        state.advance(); // -> FadeOut
        assert_eq!(state.phase(), DecidePhase::FadeOut);

        // During fadeout
        let result = state.update(400_000).unwrap();
        assert_eq!(result, StateTransition::None);

        // After fadeout
        let result = state.update(200_000).unwrap();
        assert_eq!(result, StateTransition::Next);
    }

    #[test]
    fn cancel_transitions_back() {
        let mut state = DecideState::new(make_config());
        state.create().unwrap();
        state.set_loading_complete();
        state.update(16_667).unwrap();

        state.cancel();
        assert!(state.is_cancelled());
        assert_eq!(state.phase(), DecidePhase::FadeOut);

        // After fadeout
        let result = state.update(600_000).unwrap();
        assert_eq!(result, StateTransition::Back);
    }

    #[test]
    fn stays_loading_until_complete() {
        let mut state = DecideState::new(make_config());
        state.create().unwrap();

        // Multiple updates without loading complete
        for _ in 0..10 {
            let result = state.update(100_000).unwrap();
            assert_eq!(result, StateTransition::None);
            assert_eq!(state.phase(), DecidePhase::Loading);
        }
    }

    #[test]
    fn dispose_resets_state() {
        let mut state = DecideState::new(make_config());
        state.create().unwrap();
        state.set_loading_complete();
        state.update(16_667).unwrap();
        state.advance();

        state.dispose();
        assert_eq!(state.phase(), DecidePhase::Loading);
    }
}

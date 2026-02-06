use anyhow::Result;

/// Transition result from a game state update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTransition {
    /// Stay in the current state.
    None,
    /// Transition to the next state (e.g., play -> result).
    Next,
    /// Return to the previous state (e.g., play -> select).
    Back,
}

/// Trait for game states (play, select, result, etc.).
pub trait GameState {
    /// Initialize the state. Called once when entering.
    fn create(&mut self) -> Result<()>;

    /// Update the state with elapsed time.
    /// Returns the desired state transition.
    fn update(&mut self, dt_us: i64) -> Result<StateTransition>;

    /// Clean up resources. Called once when leaving.
    fn dispose(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyState {
        created: bool,
        disposed: bool,
        update_count: u32,
    }

    impl DummyState {
        fn new() -> Self {
            Self {
                created: false,
                disposed: false,
                update_count: 0,
            }
        }
    }

    impl GameState for DummyState {
        fn create(&mut self) -> Result<()> {
            self.created = true;
            Ok(())
        }

        fn update(&mut self, _dt_us: i64) -> Result<StateTransition> {
            self.update_count += 1;
            if self.update_count >= 3 {
                Ok(StateTransition::Next)
            } else {
                Ok(StateTransition::None)
            }
        }

        fn dispose(&mut self) {
            self.disposed = true;
        }
    }

    #[test]
    fn game_state_lifecycle() {
        let mut state = DummyState::new();
        assert!(!state.created);

        state.create().unwrap();
        assert!(state.created);

        assert_eq!(state.update(16_667).unwrap(), StateTransition::None);
        assert_eq!(state.update(16_667).unwrap(), StateTransition::None);
        assert_eq!(state.update(16_667).unwrap(), StateTransition::Next);

        state.dispose();
        assert!(state.disposed);
    }
}

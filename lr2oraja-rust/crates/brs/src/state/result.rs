// Result state — stub for Phase 11.
//
// Immediately transitions to MusicSelect. Full implementation in Sub-phase E.

use tracing::info;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Result state stub.
pub struct ResultState;

impl ResultState {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResultState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for ResultState {
    fn create(&mut self, _ctx: &mut StateContext) {
        info!("Result: create (stub)");
    }

    fn render(&mut self, ctx: &mut StateContext) {
        info!("Result: stub transitioning to MusicSelect");
        *ctx.transition = Some(AppStateType::MusicSelect);
    }
}

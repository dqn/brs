// Play state — stub for Phase 11.
//
// Immediately transitions to Result. Full implementation in Sub-phase D.

use tracing::info;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Play state stub.
pub struct PlayState;

impl PlayState {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlayState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for PlayState {
    fn create(&mut self, _ctx: &mut StateContext) {
        info!("Play: create (stub)");
    }

    fn render(&mut self, ctx: &mut StateContext) {
        // Stub: immediately transition to Result
        info!("Play: stub transitioning to Result");
        *ctx.transition = Some(AppStateType::Result);
    }
}

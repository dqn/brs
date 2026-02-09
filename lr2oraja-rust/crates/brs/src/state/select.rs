// MusicSelect state — stub for Phase 11.
//
// When a BMS model is loaded (via --bms CLI), immediately transitions to Decide.
// Full implementation deferred to later sub-phases.

use tracing::info;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Music select state stub.
pub struct MusicSelectState;

impl MusicSelectState {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MusicSelectState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for MusicSelectState {
    fn create(&mut self, ctx: &mut StateContext) {
        info!("MusicSelect: create");
        // If a BMS model is already loaded (from CLI), skip to Decide
        if ctx.resource.bms_model.is_some() {
            info!("MusicSelect: BMS model loaded, transitioning to Decide");
            *ctx.transition = Some(AppStateType::Decide);
        }
    }

    fn render(&mut self, _ctx: &mut StateContext) {
        // Stub: no rendering in Phase 11
    }
}

// SkinConfig state — stub for Phase 11.

use tracing::info;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Skin configuration state stub.
pub struct SkinConfigState;

impl SkinConfigState {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SkinConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for SkinConfigState {
    fn create(&mut self, _ctx: &mut StateContext) {
        info!("SkinConfig: create (stub)");
    }

    fn render(&mut self, ctx: &mut StateContext) {
        info!("SkinConfig: stub transitioning to MusicSelect");
        *ctx.transition = Some(AppStateType::MusicSelect);
    }
}

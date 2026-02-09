// KeyConfig state — stub for Phase 11.

use tracing::info;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Key configuration state stub.
pub struct KeyConfigState;

impl KeyConfigState {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KeyConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for KeyConfigState {
    fn create(&mut self, _ctx: &mut StateContext) {
        info!("KeyConfig: create (stub)");
    }

    fn render(&mut self, ctx: &mut StateContext) {
        info!("KeyConfig: stub transitioning to MusicSelect");
        *ctx.transition = Some(AppStateType::MusicSelect);
    }
}

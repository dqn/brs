// CourseResult state — stub for Phase 11.

use tracing::info;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Course result state stub.
pub struct CourseResultState;

impl CourseResultState {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CourseResultState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for CourseResultState {
    fn create(&mut self, _ctx: &mut StateContext) {
        info!("CourseResult: create (stub)");
    }

    fn render(&mut self, ctx: &mut StateContext) {
        info!("CourseResult: stub transitioning to MusicSelect");
        *ctx.transition = Some(AppStateType::MusicSelect);
    }
}

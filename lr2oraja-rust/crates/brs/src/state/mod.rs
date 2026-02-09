// Game state handler trait and state modules.
//
// Corresponds to Java MainState abstract class.

pub mod course_result;
pub mod decide;
pub mod key_config;
pub mod play;
pub mod result;
pub mod select;
pub mod skin_config;

use crate::app_state::AppStateType;
use crate::player_resource::PlayerResource;
use crate::timer_manager::TimerManager;
use bms_config::{Config, PlayerConfig};

/// Context passed to state handlers on each callback.
pub struct StateContext<'a> {
    pub timer: &'a mut TimerManager,
    pub resource: &'a mut PlayerResource,
    #[allow(dead_code)]
    pub config: &'a Config,
    pub player_config: &'a PlayerConfig,
    /// Set this to request a state transition at the end of the frame.
    pub transition: &'a mut Option<AppStateType>,
}

/// Trait for game state handlers. Each variant of `AppStateType` has
/// a corresponding implementation.
///
/// Lifecycle: `create` -> `prepare` -> (`render` + `input`)* -> `shutdown` -> `dispose`
pub trait GameStateHandler: Send + Sync {
    /// Called when entering this state (after previous state's shutdown).
    fn create(&mut self, ctx: &mut StateContext);

    /// Called once after `create`, before the first frame.
    fn prepare(&mut self, _ctx: &mut StateContext) {}

    /// Called every frame. Update timers, check transitions.
    fn render(&mut self, ctx: &mut StateContext);

    /// Called every frame for input processing.
    fn input(&mut self, _ctx: &mut StateContext) {}

    /// Called when leaving this state (before next state's create).
    fn shutdown(&mut self, _ctx: &mut StateContext) {}

    /// Called for final cleanup (resource deallocation).
    #[allow(dead_code)]
    fn dispose(&mut self) {}
}

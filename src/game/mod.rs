mod game_state;
mod gamepad;
mod gauge;
mod input;
mod judge;
mod options;
mod result;
mod score;
mod state;

pub use game_state::GameState;
#[allow(unused_imports)]
pub use gauge::{GaugeManager, GaugeSystem, GaugeType};
pub use input::InputHandler;
// Public API for library consumers and tests
#[allow(unused_imports)]
pub use judge::{
    JudgeConfig, JudgeConfigBuilder, JudgeRank, JudgeResult, JudgeSystem, JudgeSystemType,
    ReleaseConfig, TimingDirection, TimingStats,
};
#[allow(unused_imports)]
pub use options::{
    LaneMapping, RandomOption, apply_battle, apply_legacy_note, apply_random_option, generate_seed,
};
pub use result::{ClearLamp, PlayResult};
pub use score::ScoreManager;
#[allow(unused_imports)]
pub use state::{GamePlayState, NoteState};

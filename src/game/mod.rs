mod game_state;
mod input;
mod judge;
mod result;
mod score;
mod state;

pub use game_state::GameState;
pub use input::InputHandler;
// Public API for library consumers and tests
#[allow(unused_imports)]
pub use judge::{JudgeConfig, JudgeConfigBuilder, JudgeResult, JudgeSystem, ReleaseConfig};
pub use result::PlayResult;
pub use score::ScoreManager;
#[allow(unused_imports)]
pub use state::{GamePlayState, NoteState};

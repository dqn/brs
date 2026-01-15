mod game_state;
mod gauge;
mod input;
mod judge;
mod result;
mod score;
mod state;

pub use game_state::GameState;
#[allow(unused_imports)]
pub use gauge::{GaugeManager, GaugeSystem, GaugeType};
pub use input::InputHandler;
// Public API for library consumers and tests
#[allow(unused_imports)]
pub use judge::{JudgeConfig, JudgeConfigBuilder, JudgeResult, JudgeSystem, ReleaseConfig};
pub use result::{ClearLamp, PlayResult};
pub use score::ScoreManager;
#[allow(unused_imports)]
pub use state::{GamePlayState, NoteState};

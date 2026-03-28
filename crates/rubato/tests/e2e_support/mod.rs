pub mod harness;
pub mod scenario;

pub use harness::{E2eHarness, FRAME_DURATION_US, FrameState};
pub use scenario::E2eScenario;

// Re-export commonly needed types for E2E tests
pub use rubato::audio::recording_audio_driver::AudioEvent;
pub use rubato::core::main_controller::StateCreator;
pub use rubato::skin::groove_gauge::GrooveGauge;
pub use rubato::skin::main_state_type::MainStateType;
pub use rubato::skin::score_data::ScoreData;
pub use rubato::skin::state_event::StateEvent;

//! E2E test harness for behavioral testing of the rubato application.
//!
//! Provides `E2eHarness` which wires up a `MainController` with a
//! `RecordingAudioDriver` and deterministic (frozen) timing, suitable
//! for integration tests that exercise the full state machine without
//! requiring GPU or audio hardware.

pub mod harness;
pub mod scenario;

pub use harness::{E2eHarness, FRAME_DURATION_US, FrameState};
pub use scenario::E2eScenario;

// Re-export commonly needed types for E2E tests
pub use rubato_audio::recording_audio_driver::AudioEvent;
pub use rubato::core::main_controller::StateCreator;
pub use rubato_skin::groove_gauge::GrooveGauge;
pub use rubato_skin::main_state_type::MainStateType;
pub use rubato_skin::score_data::ScoreData;
pub use rubato_skin::state_event::StateEvent;

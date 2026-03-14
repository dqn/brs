//! E2E test harness for behavioral testing of the rubato application.
//!
//! Provides `E2eHarness` which wires up a `MainController` with a
//! `RecordingAudioDriver` and deterministic (frozen) timing, suitable
//! for integration tests that exercise the full state machine without
//! requiring GPU or audio hardware.

pub mod harness;

pub use harness::{E2eHarness, FRAME_DURATION_US};

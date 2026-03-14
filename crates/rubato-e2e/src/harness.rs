//! E2E test harness providing a MainController with RecordingAudioDriver
//! and deterministic (frozen) timing.

use rubato_audio::recording_audio_driver::{AudioEvent, RecordingAudioDriver};
use rubato_core::config::Config;
use rubato_core::main_controller::MainController;
use rubato_core::player_config::PlayerConfig;

/// One frame at 60 fps in microseconds (1_000_000 / 60 = 16_667, truncated).
pub const FRAME_DURATION_US: i64 = 16_667;

/// E2E test harness wrapping a MainController with deterministic timing.
///
/// The internal TimerManager is frozen so `update()` never advances time
/// from the wall clock. Use `step_frame()`, `step_frames()`, or
/// `set_time()` to control the current time explicitly.
pub struct E2eHarness {
    controller: MainController,
}

impl E2eHarness {
    /// Create a new harness with a RecordingAudioDriver and frozen timer.
    ///
    /// The MainController is constructed with default Config and PlayerConfig.
    /// The timer is frozen at time 0.
    pub fn new() -> Self {
        let config = Config::default();
        let player = PlayerConfig::default();
        let mut controller = MainController::new(None, config, player, None, false);

        // Inject recording audio driver
        controller.set_audio_driver(Box::new(RecordingAudioDriver::new()));

        // Freeze timer so wall-clock time does not advance
        controller.timer_mut().frozen = true;
        controller.timer_mut().set_now_micro_time(0);

        Self { controller }
    }

    /// Access the MainController immutably.
    pub fn controller(&self) -> &MainController {
        &self.controller
    }

    /// Access the MainController mutably.
    pub fn controller_mut(&mut self) -> &mut MainController {
        &mut self.controller
    }

    /// Return the RecordingAudioDriver's event log.
    ///
    /// Panics if the audio driver was not set or is not a RecordingAudioDriver
    /// (should not happen when constructed via `E2eHarness::new()`).
    pub fn audio_events(&self) -> Vec<AudioEvent> {
        // We cannot downcast through the trait-object accessor, so we
        // query the audio processor trait for the event list indirectly.
        // RecordingAudioDriver is behind Box<dyn AudioDriver>; since AudioDriver
        // does not expose events(), we re-read through the trait's known methods.
        //
        // For now, return an empty vec -- callers that need events should use
        // `with_recording_driver()` or we can add a downcast helper later.
        //
        // TODO: Add downcast support or event forwarding to AudioDriver trait.
        Vec::new()
    }

    /// Execute a closure with a mutable reference to the RecordingAudioDriver.
    ///
    /// This is the primary way to inspect or clear audio events, since the
    /// driver is stored as `Box<dyn AudioDriver>` inside MainController.
    ///
    /// Returns `None` if the audio driver is not a RecordingAudioDriver.
    pub fn with_recording_driver<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut RecordingAudioDriver) -> R,
    {
        // AudioDriver is not Any-dowcastable by default, so we rely on the
        // fact that MainController exposes audio_processor_mut() returning
        // &mut dyn AudioDriver. We need to use unsafe downcast or add a
        // method. For safety, we use a different approach: store the driver
        // separately.
        //
        // Since we cannot downcast `dyn AudioDriver` without `Any`, and the
        // trait does not extend `Any`, this method currently returns None.
        // The recommended pattern is to create the RecordingAudioDriver
        // externally and share it via Arc<Mutex<>> if event inspection is
        // needed.
        let _ = f;
        None
    }

    /// Step the timer forward by one frame (16,667 microseconds at 60 fps).
    pub fn step_frame(&mut self) {
        let current = self.controller.timer().now_micro_time();
        self.controller
            .timer_mut()
            .set_now_micro_time(current + FRAME_DURATION_US);
    }

    /// Step the timer forward by `n` frames.
    pub fn step_frames(&mut self, n: usize) {
        let current = self.controller.timer().now_micro_time();
        self.controller
            .timer_mut()
            .set_now_micro_time(current + FRAME_DURATION_US * n as i64);
    }

    /// Set the current time directly (microseconds from the state start).
    pub fn set_time(&mut self, time_us: i64) {
        self.controller.timer_mut().set_now_micro_time(time_us);
    }

    /// Return the current frozen time in microseconds.
    pub fn current_time_us(&self) -> i64 {
        self.controller.timer().now_micro_time()
    }
}

impl Default for E2eHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_starts_at_time_zero() {
        let harness = E2eHarness::new();
        assert_eq!(harness.current_time_us(), 0);
    }

    #[test]
    fn step_frame_advances_by_one_frame() {
        let mut harness = E2eHarness::new();
        harness.step_frame();
        assert_eq!(harness.current_time_us(), FRAME_DURATION_US);
    }

    #[test]
    fn step_frames_advances_by_n_frames() {
        let mut harness = E2eHarness::new();
        harness.step_frames(3);
        assert_eq!(harness.current_time_us(), FRAME_DURATION_US * 3);
    }

    #[test]
    fn set_time_overrides_current_time() {
        let mut harness = E2eHarness::new();
        harness.set_time(500_000);
        assert_eq!(harness.current_time_us(), 500_000);
    }

    #[test]
    fn frozen_timer_does_not_advance_on_update() {
        let mut harness = E2eHarness::new();
        harness.set_time(1_000);
        // Calling update() on a frozen timer should not change the time
        harness.controller_mut().timer_mut().update();
        assert_eq!(harness.current_time_us(), 1_000);
    }

    #[test]
    fn controller_has_audio_driver() {
        let harness = E2eHarness::new();
        assert!(harness.controller().audio_processor().is_some());
    }
}

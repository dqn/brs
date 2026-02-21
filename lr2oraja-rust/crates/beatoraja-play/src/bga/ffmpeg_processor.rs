use crate::bga::movie_processor::MovieProcessor;
use crate::stubs::Texture;

/// Processor status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProcessorStatus {
    TextureInactive,
    TextureActive,
    Disposed,
}

/// FFmpeg-based movie processor
pub struct FFmpegProcessor {
    /// Frame display rate (1/n)
    fpsd: i32,
    time: i64,
    processor_status: ProcessorStatus,
}

/// Commands for the movie seek thread
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    Play,
    Loop,
    Stop,
    Halt,
}

impl FFmpegProcessor {
    pub fn new(fpsd: i32) -> Self {
        FFmpegProcessor {
            fpsd,
            time: 0,
            processor_status: ProcessorStatus::TextureInactive,
        }
    }

    pub fn create(&mut self, _filepath: &str) {
        // TODO: Phase 7+ dependency - requires FFmpeg bindings (e.g., ffmpeg-next crate)
        // In Java, this starts a MovieSeekThread that opens the video file
    }
}

impl MovieProcessor for FFmpegProcessor {
    fn get_frame(&mut self, time: i64) -> Option<Texture> {
        self.time = time;
        if self.processor_status == ProcessorStatus::TextureActive {
            // TODO: return actual texture
            None
        } else {
            None
        }
    }

    fn play(&mut self, time: i64, _loop_play: bool) {
        if self.processor_status == ProcessorStatus::Disposed {
            return;
        }
        self.time = time;
        // TODO: send Play/Loop command to seek thread
    }

    fn stop(&mut self) {
        if self.processor_status == ProcessorStatus::Disposed {}
        // TODO: send Stop command to seek thread
    }

    fn dispose(&mut self) {
        self.processor_status = ProcessorStatus::Disposed;
        // TODO: send Halt command to seek thread and clean up
    }
}

/// Timer observer for movie playback
pub trait TimerObserver {
    fn get_micro_time(&self) -> i64;
}

use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tracing::warn;

use crate::audio::{AudioConfig, AudioDriver, KeysoundProcessor};
use crate::database::SongData;
use crate::model::{BMSModel, load_chart};

/// Pre-loaded resources ready for PlayState.
#[derive(Debug)]
pub struct PreparedPlayData {
    pub song_data: SongData,
    pub model: BMSModel,
    pub audio_driver: AudioDriver,
    pub keysound_processor: KeysoundProcessor,
}

/// Stage of the loading process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadingStage {
    #[default]
    NotStarted,
    ParsingBms,
    LoadingAudio,
    Complete,
    Failed,
}

/// Progress information for the loading task.
#[derive(Debug, Clone, Default)]
pub struct LoadingProgress {
    pub stage: LoadingStage,
    pub bms_progress: f32,
    pub audio_progress: f32,
    pub audio_loaded: usize,
    pub audio_total: usize,
    pub error: Option<String>,
}

impl LoadingProgress {
    /// Get overall progress as a value between 0.0 and 1.0.
    pub fn overall_progress(&self) -> f32 {
        // BMS parsing is 30%, audio loading is 70%
        self.bms_progress * 0.3 + self.audio_progress * 0.7
    }
}

/// Internal state shared between main thread and worker thread.
struct LoadingTaskState {
    progress: LoadingProgress,
    result: Option<Result<PreparedPlayData>>,
}

/// Background loading task for BMS and audio resources.
pub struct LoadingTask {
    state: Arc<Mutex<LoadingTaskState>>,
}

impl LoadingTask {
    /// Start a new loading task for the given song.
    pub fn start(song_data: SongData) -> Self {
        let state = Arc::new(Mutex::new(LoadingTaskState {
            progress: LoadingProgress::default(),
            result: None,
        }));

        let state_clone = state.clone();
        let song_data_clone = song_data.clone();

        std::thread::spawn(move || {
            Self::load_resources(song_data_clone, state_clone);
        });

        Self { state }
    }

    /// Get the current loading progress.
    pub fn progress(&self) -> LoadingProgress {
        self.state.lock().unwrap().progress.clone()
    }

    /// Check if loading is complete (successfully or with error).
    pub fn is_complete(&self) -> bool {
        let state = self.state.lock().unwrap();
        matches!(
            state.progress.stage,
            LoadingStage::Complete | LoadingStage::Failed
        )
    }

    /// Take the result of the loading task.
    /// Returns None if loading is not complete or result was already taken.
    pub fn take_result(&self) -> Option<Result<PreparedPlayData>> {
        self.state.lock().unwrap().result.take()
    }

    fn load_resources(song_data: SongData, state: Arc<Mutex<LoadingTaskState>>) {
        // Stage 1: Parse BMS
        {
            let mut s = state.lock().unwrap();
            s.progress.stage = LoadingStage::ParsingBms;
        }

        let loaded = match load_chart(&song_data.path) {
            Ok(bms) => bms,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.progress.stage = LoadingStage::Failed;
                s.progress.error = Some(format!("Failed to parse BMS: {}", e));
                s.result = Some(Err(e));
                return;
            }
        };

        let model = match BMSModel::from_bms(&loaded.bms, loaded.format, Some(&song_data.path)) {
            Ok(m) => m,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.progress.stage = LoadingStage::Failed;
                s.progress.error = Some(format!("Failed to create BMS model: {}", e));
                s.result = Some(Err(e));
                return;
            }
        };

        {
            let mut s = state.lock().unwrap();
            s.progress.bms_progress = 1.0;
        }

        // Stage 2: Load Audio
        {
            let mut s = state.lock().unwrap();
            s.progress.stage = LoadingStage::LoadingAudio;
            s.progress.audio_total = model.wav_files.len();
        }

        let audio_config = AudioConfig::default();
        let mut audio_driver = match AudioDriver::new(audio_config) {
            Ok(d) => d,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.progress.stage = LoadingStage::Failed;
                s.progress.error = Some(format!("Failed to create audio driver: {}", e));
                s.result = Some(Err(e));
                return;
            }
        };

        let bms_dir = song_data.path.parent().unwrap_or(Path::new("."));

        match audio_driver.load_sounds(&model, bms_dir) {
            Ok(progress) => {
                let mut s = state.lock().unwrap();
                s.progress.audio_loaded = progress.loaded();
                s.progress.audio_progress = if progress.total() > 0 {
                    progress.loaded() as f32 / progress.total() as f32
                } else {
                    1.0
                };
            }
            Err(e) => {
                // Log but don't fail - some sounds may be missing
                warn!("Failed to load some sounds: {}", e);
            }
        }

        // Setup keysound processor
        let mut keysound_processor = KeysoundProcessor::new();
        keysound_processor.load_bgm_events(model.bgm_events.clone());

        // Complete
        {
            let mut s = state.lock().unwrap();
            s.progress.stage = LoadingStage::Complete;
            s.progress.audio_progress = 1.0;
            s.result = Some(Ok(PreparedPlayData {
                song_data,
                model,
                audio_driver,
                keysound_processor,
            }));
        }
    }
}

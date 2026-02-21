use std::collections::VecDeque;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::stubs::*;

/// Preview music processor
/// Translates: bms.player.beatoraja.select.PreviewMusicProcessor
pub struct PreviewMusicProcessor {
    /// Music loading task queue
    commands: Arc<Mutex<VecDeque<String>>>,
    preview_running: Arc<AtomicBool>,
    default_music: String,
    current: Option<SongData>,
}

impl PreviewMusicProcessor {
    pub fn new(_audio: &AudioDriver, _config: &Config) -> Self {
        Self {
            commands: Arc::new(Mutex::new(VecDeque::new())),
            preview_running: Arc::new(AtomicBool::new(false)),
            default_music: String::new(),
            current: None,
        }
    }

    pub fn set_default(&mut self, path: &str) {
        self.default_music = path.to_string();
    }

    pub fn start(&mut self, song: Option<&SongData>) {
        if !self.preview_running.load(Ordering::SeqCst) {
            self.preview_running.store(true, Ordering::SeqCst);
            // In Java: starts PreviewThread. Here we would spawn a thread.
            // Stubbed since audio playback requires runtime integration.
        }
        self.current = song.cloned();

        let mut preview_path = String::new();
        if let Some(song) = song
            && let Some(preview) = song.get_preview()
            && !preview.is_empty()
            && let Some(song_path) = song.get_path()
            && let Some(parent) = Path::new(song_path).parent()
        {
            preview_path = parent.join(preview).to_string_lossy().to_string();
        }

        if let Ok(mut cmds) = self.commands.lock() {
            cmds.push_back(preview_path);
        }
    }

    pub fn get_song_data(&self) -> Option<&SongData> {
        self.current.as_ref()
    }

    pub fn stop(&mut self) {
        self.preview_running.store(false, Ordering::SeqCst);
    }
}

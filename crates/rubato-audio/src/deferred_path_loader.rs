/// Deferred path sound loader - loads audio files on a background thread
/// to avoid blocking the render thread on cache misses.
///
/// Used by GdxSoundDriver, GdxAudioDeviceDriver, and PortAudioDriver.
use std::collections::HashSet;
use std::sync::mpsc;

use kira::sound::static_sound::StaticSoundData;

/// Result of a background path sound load.
struct PathLoadResult {
    path: String,
    sound: Option<StaticSoundData>,
}

/// A loaded sound with its path and pending play requests (volume, loop).
type LoadedSoundEntry = (String, StaticSoundData, Vec<(f32, bool)>);

/// Manages deferred (non-blocking) loading of path-based audio files.
///
/// On cache miss, spawns a background thread to load the file. The loaded
/// sound data is received via channel on the next `poll()` call and cached
/// for immediate playback.
pub(crate) struct DeferredPathLoader {
    tx: mpsc::Sender<PathLoadResult>,
    rx: mpsc::Receiver<PathLoadResult>,
    /// Paths currently being loaded in background threads.
    loading: HashSet<String>,
    /// Play requests waiting for their path to finish loading.
    pending_plays: Vec<(String, f32, bool)>,
}

impl DeferredPathLoader {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx,
            rx,
            loading: HashSet::new(),
            pending_plays: Vec::new(),
        }
    }

    /// Maximum number of concurrent background load threads.
    const MAX_CONCURRENT_LOADS: usize = 8;

    /// Queue a background load for the given path if not already loading.
    /// Also records the play request so it can be fulfilled when loading completes.
    ///
    /// When the maximum number of concurrent loads is reached and the path is
    /// not already loading, both the load and the play request are skipped.
    /// During rapid song scrolling the user has already moved past the track,
    /// so the sound would never be heard anyway. This prevents unbounded growth
    /// of `pending_plays` with entries that no background thread will ever drain.
    pub fn request_load(&mut self, path: &str, volume: f32, loop_play: bool) {
        if self.loading.contains(path) {
            // A thread is already loading this path; just record the play request
            // so it will be fulfilled when the load completes.
            self.pending_plays
                .push((path.to_string(), volume, loop_play));
            return;
        }

        if self.loading.len() >= Self::MAX_CONCURRENT_LOADS {
            // Concurrency limit reached and no thread is loading this path.
            // Skip both the load and the play request to avoid orphaned
            // pending_plays entries that would never be drained.
            return;
        }

        self.loading.insert(path.to_string());
        let tx = self.tx.clone();
        let path_owned = path.to_string();
        std::thread::Builder::new()
            .name(format!("path-audio-load:{}", path))
            .spawn(move || {
                let candidates = crate::audio_driver::paths(&path_owned);
                let mut loaded = None;
                for candidate in &candidates {
                    if let Ok(sound_data) = StaticSoundData::from_file(candidate) {
                        loaded = Some(sound_data);
                        break;
                    }
                }
                let _ = tx.send(PathLoadResult {
                    path: path_owned,
                    sound: loaded,
                });
            })
            .ok();
        self.pending_plays
            .push((path.to_string(), volume, loop_play));
    }

    /// Poll for completed background loads. Returns newly loaded sounds
    /// and their pending play requests.
    ///
    /// Caller is responsible for inserting into `path_sound_cache` and playing.
    pub fn poll(&mut self) -> Vec<LoadedSoundEntry> {
        let mut results = Vec::new();

        while let Ok(result) = self.rx.try_recv() {
            self.loading.remove(&result.path);
            if let Some(sound) = result.sound {
                // Collect all pending plays for this path
                let plays: Vec<(f32, bool)> = self
                    .pending_plays
                    .iter()
                    .filter(|(p, _, _)| *p == result.path)
                    .map(|(_, v, l)| (*v, *l))
                    .collect();
                self.pending_plays.retain(|(p, _, _)| *p != result.path);
                results.push((result.path, sound, plays));
            } else {
                // Load failed - discard pending plays for this path
                self.pending_plays.retain(|(p, _, _)| *p != result.path);
            }
        }

        results
    }

    /// Drain all pending state (e.g., on dispose).
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.loading.clear();
        self.pending_plays.clear();
        // Drain any remaining messages
        while self.rx.try_recv().is_ok() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: when the concurrency limit is reached and a new (not-already-loading)
    /// path is requested, pending_plays must NOT grow. Without the fix, orphaned entries
    /// accumulated because no background thread would ever produce a result to drain them.
    #[test]
    fn pending_plays_not_added_when_concurrency_limit_reached() {
        let mut loader = DeferredPathLoader::new();

        // Simulate MAX_CONCURRENT_LOADS paths already in flight by inserting
        // directly into `loading` (avoids spawning real threads).
        for i in 0..DeferredPathLoader::MAX_CONCURRENT_LOADS {
            loader.loading.insert(format!("already_loading_{i}"));
        }
        assert_eq!(
            loader.loading.len(),
            DeferredPathLoader::MAX_CONCURRENT_LOADS
        );

        // Request a load for a path that is NOT in the loading set.
        // The concurrency limit is full, so this should be silently dropped.
        loader.request_load("new_path_over_limit", 1.0, false);

        assert!(
            loader.pending_plays.is_empty(),
            "pending_plays should remain empty when concurrency limit is reached \
             and the path is not already loading, but had {} entries",
            loader.pending_plays.len()
        );
        // The path should not have been added to loading either.
        assert!(
            !loader.loading.contains("new_path_over_limit"),
            "path should not be added to loading set when limit is reached"
        );
    }

    /// When a path IS already loading, additional request_load calls for the same
    /// path should still add to pending_plays (the thread will eventually drain them).
    #[test]
    fn pending_plays_added_when_path_already_loading() {
        let mut loader = DeferredPathLoader::new();

        // Simulate MAX_CONCURRENT_LOADS paths in flight, one of which is "my_path".
        loader.loading.insert("my_path".to_string());
        for i in 1..DeferredPathLoader::MAX_CONCURRENT_LOADS {
            loader.loading.insert(format!("other_{i}"));
        }

        // Even though the limit is reached, "my_path" IS already loading,
        // so the play request should be recorded.
        loader.request_load("my_path", 0.8, true);

        assert_eq!(
            loader.pending_plays.len(),
            1,
            "pending_plays should have exactly 1 entry for an already-loading path"
        );
        assert_eq!(loader.pending_plays[0].0, "my_path");
        assert!((loader.pending_plays[0].1 - 0.8).abs() < f32::EPSILON);
        assert!(loader.pending_plays[0].2);
    }
}

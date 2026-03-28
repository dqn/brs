//! RAII guard that restores the process CWD on drop.
//!
//! Prevents CWD corruption when a test panics after `set_current_dir`.

use std::path::{Path, PathBuf};

/// RAII guard that saves the current working directory and restores it on drop.
///
/// # Usage
///
/// ```ignore
/// let _cwd = CurrentDirGuard::set(some_dir);
/// // ... test code that needs CWD to be `some_dir` ...
/// // CWD is restored automatically when `_cwd` is dropped, even on panic.
/// ```
///
/// Combine with a `Mutex` when multiple tests may change CWD concurrently.
pub struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    /// Changes CWD to `dir` and returns a guard that restores the original on drop.
    pub fn set(dir: &Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir).unwrap();
        Self { original }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

// Download task status tracking
//
// Corresponds to Java DownloadTask.java / DownloadTaskStatus enum.

use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Status of a download task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadTaskStatus {
    Prepare,
    Downloading,
    Downloaded,
    Extracted,
    Error,
    Cancel,
}

impl DownloadTaskStatus {
    /// Whether this status is a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Extracted | Self::Error | Self::Cancel)
    }
}

/// A single download task.
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub id: usize,
    pub url: String,
    pub name: String,
    pub hash: String,
    pub status: DownloadTaskStatus,
    pub download_size: u64,
    pub content_length: u64,
    pub error_message: Option<String>,
    pub time_finished: Option<Instant>,
}

impl DownloadTask {
    pub fn new(id: usize, url: String, name: String, hash: String) -> Self {
        Self {
            id,
            url,
            name,
            hash,
            status: DownloadTaskStatus::Prepare,
            download_size: 0,
            content_length: 0,
            error_message: None,
            time_finished: None,
        }
    }

    /// Update the task status.
    /// Automatically sets `time_finished` when reaching a terminal state.
    pub fn set_status(&mut self, status: DownloadTaskStatus) {
        self.status = status;
        if status.is_terminal() {
            self.time_finished = Some(Instant::now());
        }
    }

    /// Set the error status with a message.
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.set_status(DownloadTaskStatus::Error);
    }

    /// Download progress as a fraction (0.0 to 1.0).
    /// Returns 0.0 if content_length is unknown (0).
    pub fn progress(&self) -> f64 {
        if self.content_length == 0 {
            0.0
        } else {
            self.download_size as f64 / self.content_length as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        let task = DownloadTask::new(1, "http://example.com".into(), "test".into(), "abc".into());
        assert_eq!(task.id, 1);
        assert_eq!(task.status, DownloadTaskStatus::Prepare);
        assert_eq!(task.download_size, 0);
        assert_eq!(task.content_length, 0);
        assert!(task.error_message.is_none());
        assert!(task.time_finished.is_none());
    }

    #[test]
    fn test_status_transitions() {
        let mut task =
            DownloadTask::new(1, "http://example.com".into(), "test".into(), "abc".into());

        // Prepare -> Downloading
        task.set_status(DownloadTaskStatus::Downloading);
        assert_eq!(task.status, DownloadTaskStatus::Downloading);
        assert!(task.time_finished.is_none());

        // Downloading -> Downloaded
        task.set_status(DownloadTaskStatus::Downloaded);
        assert_eq!(task.status, DownloadTaskStatus::Downloaded);
        assert!(task.time_finished.is_none());

        // Downloaded -> Extracted (terminal)
        task.set_status(DownloadTaskStatus::Extracted);
        assert_eq!(task.status, DownloadTaskStatus::Extracted);
        assert!(task.time_finished.is_some());
    }

    #[test]
    fn test_error_sets_time_finished() {
        let mut task =
            DownloadTask::new(1, "http://example.com".into(), "test".into(), "abc".into());
        task.set_error("connection timeout".into());
        assert_eq!(task.status, DownloadTaskStatus::Error);
        assert_eq!(task.error_message.as_deref(), Some("connection timeout"));
        assert!(task.time_finished.is_some());
    }

    #[test]
    fn test_cancel_sets_time_finished() {
        let mut task =
            DownloadTask::new(1, "http://example.com".into(), "test".into(), "abc".into());
        task.set_status(DownloadTaskStatus::Cancel);
        assert!(task.time_finished.is_some());
    }

    #[test]
    fn test_progress() {
        let mut task =
            DownloadTask::new(1, "http://example.com".into(), "test".into(), "abc".into());

        // Unknown content length
        assert_eq!(task.progress(), 0.0);

        // Partial download
        task.content_length = 1000;
        task.download_size = 500;
        assert!((task.progress() - 0.5).abs() < f64::EPSILON);

        // Complete
        task.download_size = 1000;
        assert!((task.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_is_terminal() {
        assert!(!DownloadTaskStatus::Prepare.is_terminal());
        assert!(!DownloadTaskStatus::Downloading.is_terminal());
        assert!(!DownloadTaskStatus::Downloaded.is_terminal());
        assert!(DownloadTaskStatus::Extracted.is_terminal());
        assert!(DownloadTaskStatus::Error.is_terminal());
        assert!(DownloadTaskStatus::Cancel.is_terminal());
    }
}

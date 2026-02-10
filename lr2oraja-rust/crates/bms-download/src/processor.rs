// Download processor
//
// Manages download tasks with concurrent execution.
// Corresponds to Java HttpDownloadProcessor.java.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::anyhow;
use tokio::sync::{Mutex, Semaphore};
use tracing::{info, warn};

use crate::extract;
use crate::task::{DownloadTask, DownloadTaskStatus};

/// Default maximum concurrent downloads.
const DEFAULT_MAX_CONCURRENT: usize = 5;

/// Manages download tasks with concurrent execution limits.
pub struct HttpDownloadProcessor {
    tasks: Arc<Mutex<Vec<DownloadTask>>>,
    id_counter: AtomicUsize,
    semaphore: Arc<Semaphore>,
    download_dir: PathBuf,
    pub max_concurrent: usize,
}

impl HttpDownloadProcessor {
    pub fn new(download_dir: impl Into<PathBuf>) -> Self {
        Self::with_max_concurrent(download_dir, DEFAULT_MAX_CONCURRENT)
    }

    pub fn with_max_concurrent(download_dir: impl Into<PathBuf>, max_concurrent: usize) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            id_counter: AtomicUsize::new(0),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            download_dir: download_dir.into(),
            max_concurrent,
        }
    }

    /// Add a new download task. Returns the task ID.
    pub async fn add_task(&self, url: String, name: String, hash: String) -> usize {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let task = DownloadTask::new(id, url, name, hash);
        self.tasks.lock().await.push(task);
        id
    }

    /// Start downloading a task by ID.
    /// Spawns a tokio task that respects the concurrency semaphore.
    pub fn start_download(&self, task_id: usize) {
        let tasks = self.tasks.clone();
        let semaphore = self.semaphore.clone();
        let download_dir = self.download_dir.clone();

        tokio::spawn(async move {
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    warn!("Semaphore closed for task {}", task_id);
                    return;
                }
            };

            // Update status to Downloading
            {
                let mut tasks = tasks.lock().await;
                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                    task.set_status(DownloadTaskStatus::Downloading);
                } else {
                    warn!("Task {} not found", task_id);
                    return;
                }
            }

            // Get the URL
            let url = {
                let tasks = tasks.lock().await;
                match tasks.iter().find(|t| t.id == task_id) {
                    Some(task) => task.url.clone(),
                    None => return,
                }
            };

            // Download
            let client = reqwest::Client::new();
            match download_file(&client, &url, &tasks, task_id).await {
                Ok(archive_path) => {
                    // Update status to Downloaded
                    {
                        let mut tasks = tasks.lock().await;
                        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                            task.set_status(DownloadTaskStatus::Downloaded);
                        }
                    }

                    // Extract
                    match extract::detect_and_extract(&archive_path, &download_dir) {
                        Ok(()) => {
                            let mut tasks = tasks.lock().await;
                            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                task.set_status(DownloadTaskStatus::Extracted);
                            }
                            info!("Task {} extracted successfully", task_id);

                            // Clean up archive
                            if let Err(e) = tokio::fs::remove_file(&archive_path).await {
                                warn!("Failed to remove archive {:?}: {}", archive_path, e);
                            }
                        }
                        Err(e) => {
                            let mut tasks = tasks.lock().await;
                            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                                task.set_error(format!("extraction failed: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    let mut tasks = tasks.lock().await;
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                        task.set_error(format!("download failed: {}", e));
                    }
                }
            }
        });
    }

    /// Cancel a task by ID.
    pub async fn cancel_task(&self, task_id: usize) {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.set_status(DownloadTaskStatus::Cancel);
        }
    }

    /// Get a snapshot of all tasks.
    pub async fn get_tasks(&self) -> Vec<DownloadTask> {
        self.tasks.lock().await.clone()
    }

    /// Get a snapshot of a single task by ID.
    pub async fn get_task(&self, task_id: usize) -> Option<DownloadTask> {
        self.tasks
            .lock()
            .await
            .iter()
            .find(|t| t.id == task_id)
            .cloned()
    }
}

/// Download a file from the given URL, streaming to disk.
/// Updates download_size and content_length on the task as data arrives.
async fn download_file(
    client: &reqwest::Client,
    url: &str,
    tasks: &Arc<Mutex<Vec<DownloadTask>>>,
    task_id: usize,
) -> anyhow::Result<PathBuf> {
    use tokio::io::AsyncWriteExt;

    let resp = client.get(url).send().await?.error_for_status()?;

    let content_length = resp.content_length().unwrap_or(0);

    // Determine filename from Content-Disposition header or URL
    let filename = resp
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .and_then(extract_filename_from_header)
        .unwrap_or_else(|| url.rsplit('/').next().unwrap_or("download").to_string());

    // Update content length
    {
        let mut tasks = tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.content_length = content_length;
        }
    }

    // Determine download directory from the task
    let download_dir = {
        let tasks = tasks.lock().await;
        tasks
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.hash.clone())
            .ok_or_else(|| anyhow!("task {} not found", task_id))?
    };
    let _ = download_dir; // hash is available if needed

    // Stream to file using chunk-based reading (avoids futures_util dependency)
    let file_path = std::env::temp_dir().join(&filename);
    let mut file = tokio::fs::File::create(&file_path).await?;
    let mut downloaded: u64 = 0;
    let mut resp = resp;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        // Update progress
        let mut tasks = tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.download_size = downloaded;
            // Check for cancellation
            if task.status == DownloadTaskStatus::Cancel {
                drop(tasks);
                let _ = tokio::fs::remove_file(&file_path).await;
                anyhow::bail!("download cancelled");
            }
        }
    }

    file.flush().await?;

    Ok(file_path)
}

/// Extract filename from Content-Disposition header value.
fn extract_filename_from_header(header: &str) -> Option<String> {
    // Match: filename="name.ext" or filename=name.ext
    let filename_prefix = "filename=";
    let pos = header.find(filename_prefix)?;
    let value = &header[pos + filename_prefix.len()..];
    let value = value.trim();
    if let Some(stripped) = value.strip_prefix('"') {
        // Quoted value
        let end = stripped.find('"')?;
        Some(stripped[..end].to_string())
    } else {
        // Unquoted value - take until whitespace or semicolon
        let end = value
            .find(|c: char| c.is_whitespace() || c == ';')
            .unwrap_or(value.len());
        Some(value[..end].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename_quoted() {
        let header = r#"attachment; filename="test.7z""#;
        assert_eq!(extract_filename_from_header(header), Some("test.7z".into()));
    }

    #[test]
    fn test_extract_filename_unquoted() {
        let header = "attachment; filename=test.7z";
        assert_eq!(extract_filename_from_header(header), Some("test.7z".into()));
    }

    #[test]
    fn test_extract_filename_with_semicolon() {
        let header = "attachment; filename=test.7z; size=12345";
        assert_eq!(extract_filename_from_header(header), Some("test.7z".into()));
    }

    #[test]
    fn test_extract_filename_none() {
        let header = "inline";
        assert!(extract_filename_from_header(header).is_none());
    }

    #[tokio::test]
    async fn test_add_task() {
        let tmp = tempfile::tempdir().unwrap();
        let processor = HttpDownloadProcessor::new(tmp.path());

        let id = processor
            .add_task(
                "http://example.com/test.7z".into(),
                "test song".into(),
                "abc123".into(),
            )
            .await;
        assert_eq!(id, 0);

        let tasks = processor.get_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].url, "http://example.com/test.7z");
        assert_eq!(tasks[0].name, "test song");
        assert_eq!(tasks[0].hash, "abc123");
        assert_eq!(tasks[0].status, DownloadTaskStatus::Prepare);
    }

    #[tokio::test]
    async fn test_add_multiple_tasks() {
        let tmp = tempfile::tempdir().unwrap();
        let processor = HttpDownloadProcessor::new(tmp.path());

        let id1 = processor
            .add_task("http://a.com".into(), "song1".into(), "h1".into())
            .await;
        let id2 = processor
            .add_task("http://b.com".into(), "song2".into(), "h2".into())
            .await;

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);

        let tasks = processor.get_tasks().await;
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let tmp = tempfile::tempdir().unwrap();
        let processor = HttpDownloadProcessor::new(tmp.path());

        let id = processor
            .add_task("http://example.com".into(), "test".into(), "hash".into())
            .await;
        processor.cancel_task(id).await;

        let task = processor.get_task(id).await.unwrap();
        assert_eq!(task.status, DownloadTaskStatus::Cancel);
        assert!(task.time_finished.is_some());
    }

    #[tokio::test]
    async fn test_get_nonexistent_task() {
        let tmp = tempfile::tempdir().unwrap();
        let processor = HttpDownloadProcessor::new(tmp.path());

        assert!(processor.get_task(999).await.is_none());
    }

    #[test]
    fn test_default_max_concurrent() {
        let tmp = tempfile::tempdir().unwrap();
        let processor = HttpDownloadProcessor::new(tmp.path());
        assert_eq!(processor.max_concurrent, DEFAULT_MAX_CONCURRENT);
    }

    #[test]
    fn test_custom_max_concurrent() {
        let tmp = tempfile::tempdir().unwrap();
        let processor = HttpDownloadProcessor::with_max_concurrent(tmp.path(), 10);
        assert_eq!(processor.max_concurrent, 10);
    }
}

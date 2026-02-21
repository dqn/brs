use std::sync::atomic::{AtomicI64, Ordering};

/// Corresponds to DownloadTask.DownloadTaskStatus in Java
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadTaskStatus {
    Prepare,
    Downloading,
    Downloaded,
    Extracted,
    Error,
    Cancel,
}

impl DownloadTaskStatus {
    pub fn value(&self) -> i32 {
        match self {
            DownloadTaskStatus::Prepare => 0,
            DownloadTaskStatus::Downloading => 1,
            DownloadTaskStatus::Downloaded => 2,
            DownloadTaskStatus::Extracted => 3,
            DownloadTaskStatus::Error => 4,
            DownloadTaskStatus::Cancel => 5,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DownloadTaskStatus::Prepare => "Prepare",
            DownloadTaskStatus::Downloading => "Downloading",
            DownloadTaskStatus::Downloaded => "Downloaded",
            DownloadTaskStatus::Extracted => "Finished",
            DownloadTaskStatus::Error => "Error",
            DownloadTaskStatus::Cancel => "Cancel",
        }
    }
}

/// Corresponds to DownloadTask in Java
pub struct DownloadTask {
    id: i32,
    url: String,
    name: String,
    hash: String,
    download_task_status: DownloadTaskStatus,
    download_size: i64,
    content_length: i64,
    error_message: Option<String>,
    time_finished: AtomicI64,
}

impl DownloadTask {
    pub fn new(id: i32, url: String, name: String, hash: String) -> Self {
        DownloadTask {
            id,
            url,
            name,
            hash,
            download_task_status: DownloadTaskStatus::Prepare,
            download_size: 0,
            content_length: 0,
            error_message: None,
            time_finished: AtomicI64::new(0),
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn get_download_task_status(&self) -> DownloadTaskStatus {
        self.download_task_status
    }

    pub fn set_download_task_status(&mut self, status: DownloadTaskStatus) {
        if status.value() >= DownloadTaskStatus::Extracted.value() {
            // Java: System.nanoTime()
            // Use std::time::Instant elapsed as nanos approximation
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64;
            self.time_finished.store(now, Ordering::Release);
        }
        self.download_task_status = status;
    }

    pub fn get_download_size(&self) -> i64 {
        self.download_size
    }

    pub fn set_download_size(&mut self, download_size: i64) {
        self.download_size = download_size;
    }

    pub fn get_content_length(&self) -> i64 {
        self.content_length
    }

    pub fn set_content_length(&mut self, content_length: i64) {
        self.content_length = content_length;
    }

    pub fn get_error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn set_error_message(&mut self, error_message: String) {
        self.error_message = Some(error_message);
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_time_finished(&self) -> i64 {
        self.time_finished.load(Ordering::Acquire)
    }
}

use crate::database::models::ScanResult;
use crate::database::{Database, SongDatabaseAccessor, SongScanner};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Stage of the song scan process.
/// 曲スキャン処理の段階。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanStage {
    Pending,
    Scanning,
    Complete,
    Failed,
}

/// Progress information for song scanning.
/// 曲スキャン進行情報。
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub stage: ScanStage,
    pub current_folder: Option<String>,
    pub current_file: Option<String>,
    pub scanned_files: usize,
    pub added: usize,
    pub updated: usize,
    pub deleted: usize,
    pub errors: usize,
    pub message: Option<String>,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            stage: ScanStage::Pending,
            current_folder: None,
            current_file: None,
            scanned_files: 0,
            added: 0,
            updated: 0,
            deleted: 0,
            errors: 0,
            message: None,
        }
    }
}

struct ScanTaskState {
    progress: ScanProgress,
    result: Option<Result<ScanResult>>,
}

/// Background task to scan song folders and update the song database.
/// 曲フォルダをスキャンして曲DBを更新するバックグラウンドタスク。
pub struct SongScanTask {
    state: Arc<Mutex<ScanTaskState>>,
}

impl SongScanTask {
    /// Start scanning the given folders and update the database.
    /// 指定フォルダのスキャンを開始し、DBを更新する。
    pub fn start(db_path: PathBuf, folders: Vec<PathBuf>) -> Self {
        let state = Arc::new(Mutex::new(ScanTaskState {
            progress: ScanProgress::default(),
            result: None,
        }));

        let state_clone = state.clone();
        std::thread::spawn(move || {
            Self::scan_folders(db_path, folders, state_clone);
        });

        Self { state }
    }

    /// Get the current scan progress.
    /// 現在のスキャン進行情報を取得する。
    pub fn progress(&self) -> ScanProgress {
        self.state.lock().unwrap().progress.clone()
    }

    /// Check whether the scan has completed.
    /// スキャンが完了したかを確認する。
    pub fn is_complete(&self) -> bool {
        let state = self.state.lock().unwrap();
        matches!(state.progress.stage, ScanStage::Complete | ScanStage::Failed)
    }

    /// Take the scan result, if available.
    /// スキャン結果を取得する（取得後は空になる）。
    pub fn take_result(&self) -> Option<Result<ScanResult>> {
        self.state.lock().unwrap().result.take()
    }

    fn scan_folders(db_path: PathBuf, folders: Vec<PathBuf>, state: Arc<Mutex<ScanTaskState>>) {
        if folders.is_empty() {
            let mut s = state.lock().unwrap();
            s.progress.stage = ScanStage::Failed;
            s.progress.message =
                Some("No song folders configured. / 曲フォルダが設定されていません。".to_string());
            s.result = Some(Err(anyhow!(
                "No song folders configured / 曲フォルダが設定されていません"
            )));
            return;
        }

        {
            let mut s = state.lock().unwrap();
            s.progress.stage = ScanStage::Scanning;
            s.progress.message = Some("Scanning songs... / 曲をスキャン中...".to_string());
        }

        let db = match Database::open_song_db(&db_path) {
            Ok(db) => db,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.progress.stage = ScanStage::Failed;
                s.progress.message = Some(format!(
                    "Failed to open song DB: {} / 曲DBを開けませんでした: {}",
                    e, e
                ));
                s.result = Some(Err(e));
                return;
            }
        };

        let accessor = SongDatabaseAccessor::new(&db);
        let mut summary = ScanResult::new();

        for folder in folders {
            let folder_display = folder.to_string_lossy().to_string();
            {
                let mut s = state.lock().unwrap();
                s.progress.current_folder = Some(folder_display.clone());
                s.progress.current_file = None;
            }

            if !folder.exists() {
                summary.errors.push((
                    folder.clone(),
                    "Folder not found / フォルダが見つかりません".to_string(),
                ));
                let mut s = state.lock().unwrap();
                s.progress.errors = summary.errors.len();
                continue;
            }

            let scanner = SongScanner::new(folder.clone());
            let result = scanner.scan_folder(&accessor, |msg| {
                let trimmed = msg.strip_prefix("Scanning: ").unwrap_or(msg);
                let mut s = state.lock().unwrap();
                s.progress.current_file = Some(trimmed.to_string());
                s.progress.scanned_files += 1;
            });

            match result {
                Ok(result) => {
                    summary.added += result.added;
                    summary.updated += result.updated;
                    summary.deleted += result.deleted;
                    summary.unchanged += result.unchanged;
                    summary.errors.extend(result.errors);
                }
                Err(e) => {
                    summary.errors.push((folder.clone(), e.to_string()));
                }
            }

            let mut s = state.lock().unwrap();
            s.progress.added = summary.added;
            s.progress.updated = summary.updated;
            s.progress.deleted = summary.deleted;
            s.progress.errors = summary.errors.len();
        }

        let mut s = state.lock().unwrap();
        s.progress.stage = ScanStage::Complete;
        s.progress.message = Some("Scan completed. / スキャン完了。".to_string());
        s.result = Some(Ok(summary));
    }
}

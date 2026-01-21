use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

use super::SavedScore;

/// JSON-based score repository with backup and atomic write support
pub struct ScoreRepository {
    data_dir: PathBuf,
    scores: HashMap<String, SavedScore>,
}

/// Result of loading scores from disk
#[derive(Debug)]
pub struct LoadResult {
    /// Whether loading was successful
    #[allow(dead_code)]
    pub success: bool,
    /// Whether data was recovered from backup
    #[allow(dead_code)]
    pub recovered_from_backup: bool,
    /// Error message if any
    pub error_message: Option<String>,
}

impl ScoreRepository {
    /// Create a new score repository
    pub fn new() -> Result<Self> {
        let data_dir = Self::get_data_dir()?;
        fs::create_dir_all(&data_dir)?;

        let mut repo = Self {
            data_dir,
            scores: HashMap::new(),
        };

        let load_result = repo.load()?;
        if let Some(msg) = load_result.error_message {
            eprintln!("Score repository warning: {}", msg);
        }

        Ok(repo)
    }

    /// Get the data directory for the application
    fn get_data_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "brs", "brs") {
            Ok(proj_dirs.data_dir().to_path_buf())
        } else {
            // Fallback to current directory
            Ok(PathBuf::from(".brs-data"))
        }
    }

    /// Get the path to the scores file
    fn scores_file(&self) -> PathBuf {
        self.data_dir.join("scores.json")
    }

    /// Get the path to the backup file
    fn backup_file(&self) -> PathBuf {
        self.data_dir.join("scores.json.bak")
    }

    /// Get the path to the temp file for atomic write
    fn temp_file(&self) -> PathBuf {
        self.data_dir.join("scores.json.tmp")
    }

    /// Get the path to the corrupted file backup
    fn corrupted_file(&self) -> PathBuf {
        self.data_dir.join("scores.json.corrupted")
    }

    /// Load scores from disk with error recovery
    fn load(&mut self) -> Result<LoadResult> {
        let path = self.scores_file();
        let backup_path = self.backup_file();

        // If main file doesn't exist, try to recover from backup
        if !path.exists() {
            if backup_path.exists() {
                return self.try_load_from_backup();
            }
            return Ok(LoadResult {
                success: true,
                recovered_from_backup: false,
                error_message: None,
            });
        }

        // Try to load from main file
        match self.try_load_from_path(&path) {
            Ok(scores) => {
                self.scores = scores;
                Ok(LoadResult {
                    success: true,
                    recovered_from_backup: false,
                    error_message: None,
                })
            }
            Err(e) => {
                // Main file is corrupted - backup it and try to recover
                let corrupted_path = self.corrupted_file();
                if let Err(copy_err) = fs::copy(&path, &corrupted_path) {
                    eprintln!("Warning: Failed to backup corrupted file: {}", copy_err);
                } else {
                    eprintln!(
                        "Corrupted scores file backed up to: {}",
                        corrupted_path.display()
                    );
                }

                // Try to recover from backup
                if backup_path.exists() {
                    match self.try_load_from_backup() {
                        Ok(mut result) => {
                            result.error_message = Some(format!(
                                "Main file corrupted ({}), recovered from backup",
                                e
                            ));
                            Ok(result)
                        }
                        Err(backup_err) => {
                            // Both files are corrupted - start fresh
                            eprintln!("Both main and backup files corrupted. Starting fresh.");
                            self.scores = HashMap::new();
                            Ok(LoadResult {
                                success: false,
                                recovered_from_backup: false,
                                error_message: Some(format!(
                                    "Both files corrupted: main={}, backup={}",
                                    e, backup_err
                                )),
                            })
                        }
                    }
                } else {
                    // No backup available - start fresh
                    self.scores = HashMap::new();
                    Ok(LoadResult {
                        success: false,
                        recovered_from_backup: false,
                        error_message: Some(format!(
                            "Main file corrupted and no backup available: {}",
                            e
                        )),
                    })
                }
            }
        }
    }

    /// Try to load scores from a specific path
    fn try_load_from_path(&self, path: &PathBuf) -> Result<HashMap<String, SavedScore>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        let scores: HashMap<String, SavedScore> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;
        Ok(scores)
    }

    /// Try to load from backup file
    fn try_load_from_backup(&mut self) -> Result<LoadResult> {
        let backup_path = self.backup_file();
        let scores = self.try_load_from_path(&backup_path)?;
        self.scores = scores;
        eprintln!("Recovered scores from backup file");
        Ok(LoadResult {
            success: true,
            recovered_from_backup: true,
            error_message: None,
        })
    }

    /// Save scores to disk with atomic write and backup
    pub fn save(&self) -> Result<()> {
        let path = self.scores_file();
        let temp_path = self.temp_file();
        let backup_path = self.backup_file();

        // Serialize to JSON
        let content =
            serde_json::to_string_pretty(&self.scores).context("Failed to serialize scores")?;

        // Write to temp file first
        fs::write(&temp_path, &content)
            .with_context(|| format!("Failed to write temp file: {}", temp_path.display()))?;

        // Verify the temp file is valid JSON before proceeding
        let verify_content = fs::read_to_string(&temp_path)
            .context("Failed to read back temp file for verification")?;
        let _: HashMap<String, SavedScore> = serde_json::from_str(&verify_content)
            .context("Temp file verification failed - JSON is invalid")?;

        // Backup existing file if it exists
        if path.exists() {
            if let Err(e) = fs::copy(&path, &backup_path) {
                eprintln!("Warning: Failed to create backup: {}", e);
                // Continue anyway - the temp file is valid
            }
        }

        // Atomic rename: temp -> main
        fs::rename(&temp_path, &path)
            .with_context(|| format!("Failed to rename temp file to: {}", path.display()))?;

        Ok(())
    }

    /// Get score for a chart hash
    #[allow(dead_code)]
    pub fn get(&self, hash: &str) -> Option<&SavedScore> {
        self.scores.get(hash)
    }

    /// Update score for a chart hash, returns true if it was a new best
    pub fn update(&mut self, hash: &str, new_score: SavedScore) -> bool {
        if let Some(existing) = self.scores.get_mut(hash) {
            existing.update(&new_score)
        } else {
            // First play
            let mut score = SavedScore::new(hash.to_string());
            score.update(&new_score);
            self.scores.insert(hash.to_string(), score);
            true
        }
    }

    /// Get all scores
    #[allow(dead_code)]
    pub fn all(&self) -> &HashMap<String, SavedScore> {
        &self.scores
    }

    /// Get score count
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.scores.len()
    }
}

impl Default for ScoreRepository {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to initialize ScoreRepository: {}. Using fallback.",
                e
            );
            Self {
                data_dir: PathBuf::from(".brs-data"),
                scores: HashMap::new(),
            }
        })
    }
}

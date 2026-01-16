use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;

use super::SavedScore;

/// JSON-based score repository
pub struct ScoreRepository {
    data_dir: PathBuf,
    scores: HashMap<String, SavedScore>,
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
        repo.load()?;
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

    /// Load scores from disk
    fn load(&mut self) -> Result<()> {
        let path = self.scores_file();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            self.scores = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// Save scores to disk
    pub fn save(&self) -> Result<()> {
        let path = self.scores_file();
        let content = serde_json::to_string_pretty(&self.scores)?;
        fs::write(&path, content)?;
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
        Self::new().unwrap_or_else(|_| Self {
            data_dir: PathBuf::from(".brs-data"),
            scores: HashMap::new(),
        })
    }
}

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::dan::DanGrade;

/// Record of a dan certification clear
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanRecord {
    /// Grade that was cleared
    pub grade: DanGrade,
    /// Name of the course
    pub course_name: String,
    /// Best EX score
    pub ex_score: u32,
    /// Best max combo
    pub max_combo: u32,
    /// Best judgment counts (from best EX score play)
    pub pgreat_count: u32,
    pub great_count: u32,
    pub good_count: u32,
    pub bad_count: u32,
    pub poor_count: u32,
    /// Total clear count
    pub clear_count: u32,
    /// Last cleared timestamp (Unix epoch seconds)
    pub last_cleared: u64,
}

/// Key for storing dan records (combines grade and course name)
fn record_key(grade: &DanGrade, course_name: &str) -> String {
    format!("{:?}:{}", grade, course_name)
}

/// JSON-based dan record repository
pub struct DanRepository {
    data_dir: PathBuf,
    records: HashMap<String, DanRecord>,
}

impl DanRepository {
    /// Create a new dan repository
    pub fn new() -> Result<Self> {
        let data_dir = Self::get_data_dir()?;
        fs::create_dir_all(&data_dir)?;

        let mut repo = Self {
            data_dir,
            records: HashMap::new(),
        };
        repo.load()?;
        Ok(repo)
    }

    /// Get the data directory for the application
    fn get_data_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "bms-rs", "bms-player") {
            Ok(proj_dirs.data_dir().to_path_buf())
        } else {
            // Fallback to current directory
            Ok(PathBuf::from(".bms-player-data"))
        }
    }

    /// Get the path to the dan records file
    fn records_file(&self) -> PathBuf {
        self.data_dir.join("dan_records.json")
    }

    /// Load records from disk
    fn load(&mut self) -> Result<()> {
        let path = self.records_file();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            self.records = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// Save records to disk
    pub fn save(&self) -> Result<()> {
        let path = self.records_file();
        let content = serde_json::to_string_pretty(&self.records)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Get record for a specific grade and course
    #[allow(dead_code)]
    pub fn get(&self, grade: &DanGrade, course_name: &str) -> Option<&DanRecord> {
        let key = record_key(grade, course_name);
        self.records.get(&key)
    }

    /// Update record, returns true if it was a new record (first clear or better EX score)
    pub fn update(&mut self, new_record: DanRecord) -> bool {
        let key = record_key(&new_record.grade, &new_record.course_name);

        if let Some(existing) = self.records.get_mut(&key) {
            existing.clear_count += 1;
            existing.last_cleared = new_record.last_cleared;

            // Update if better EX score
            if new_record.ex_score > existing.ex_score {
                existing.ex_score = new_record.ex_score;
                existing.pgreat_count = new_record.pgreat_count;
                existing.great_count = new_record.great_count;
                existing.good_count = new_record.good_count;
                existing.bad_count = new_record.bad_count;
                existing.poor_count = new_record.poor_count;
                return true;
            }

            // Update max combo if better
            if new_record.max_combo > existing.max_combo {
                existing.max_combo = new_record.max_combo;
            }

            false
        } else {
            // First clear
            self.records.insert(key, new_record);
            true
        }
    }

    /// Get all cleared grades
    pub fn cleared_grades(&self) -> HashSet<DanGrade> {
        self.records.values().map(|r| r.grade).collect()
    }

    /// Get the highest cleared grade
    #[allow(dead_code)]
    pub fn highest_grade(&self) -> Option<DanGrade> {
        self.records.values().map(|r| r.grade).max()
    }

    /// Get all records
    #[allow(dead_code)]
    pub fn all(&self) -> &HashMap<String, DanRecord> {
        &self.records
    }

    /// Get record count
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.records.len()
    }
}

impl Default for DanRepository {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            data_dir: PathBuf::from(".bms-player-data"),
            records: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_key() {
        assert_eq!(record_key(&DanGrade::Dan(1), "SP 初段"), "Dan(1):SP 初段");
        assert_eq!(record_key(&DanGrade::Kaiden, "SP 皆伝"), "Kaiden:SP 皆伝");
    }

    #[test]
    fn test_dan_record_serialization() {
        let record = DanRecord {
            grade: DanGrade::Dan(5),
            course_name: "SP 五段".to_string(),
            ex_score: 5000,
            max_combo: 1000,
            pgreat_count: 2000,
            great_count: 500,
            good_count: 50,
            bad_count: 10,
            poor_count: 5,
            clear_count: 3,
            last_cleared: 1234567890,
        };

        let json = serde_json::to_string(&record).unwrap();
        let parsed: DanRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.grade, DanGrade::Dan(5));
        assert_eq!(parsed.ex_score, 5000);
        assert_eq!(parsed.clear_count, 3);
    }
}

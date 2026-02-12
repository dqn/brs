// Rival player data management.
//
// Manages rival databases stored in a directory, each as a separate SQLite file.
// Provides loading, querying, and creation of rival score databases.
//
// Ported from Java RivalDataAccessor.java.

use std::path::{Path, PathBuf};

use anyhow::Result;
use bms_rule::ScoreData;
use rusqlite::Connection;

use crate::player_info::PlayerInformation;
use crate::schema::{INFO_TABLE, SCORE_TABLE, ensure_table};
use crate::score_database::score_data_from_row;

/// Manages rival player data and score caches.
#[derive(Debug)]
pub struct RivalDataAccessor {
    rival_dir: PathBuf,
    rivals: Vec<RivalEntry>,
}

/// A rival entry associating player info with a database path.
#[derive(Debug)]
pub struct RivalEntry {
    pub info: PlayerInformation,
    pub db_path: PathBuf,
}

impl RivalDataAccessor {
    pub fn new(rival_dir: impl Into<PathBuf>) -> Result<Self> {
        let rival_dir = rival_dir.into();
        if !rival_dir.exists() {
            std::fs::create_dir_all(&rival_dir)?;
        }
        Ok(Self {
            rival_dir,
            rivals: Vec::new(),
        })
    }

    pub fn rival_count(&self) -> usize {
        self.rivals.len()
    }

    pub fn get_rival(&self, index: usize) -> Option<&RivalEntry> {
        self.rivals.get(index)
    }

    pub fn rivals(&self) -> &[RivalEntry] {
        &self.rivals
    }

    /// Load all rival databases from the rival directory.
    pub fn load_local_rivals(&mut self) -> Result<()> {
        self.rivals.clear();
        if !self.rival_dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&self.rival_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "db")
                && let Ok(info) = Self::read_rival_info(&path)
            {
                self.rivals.push(RivalEntry {
                    info,
                    db_path: path,
                });
            }
        }
        Ok(())
    }

    /// Read player information from a rival database.
    fn read_rival_info(db_path: &Path) -> Result<PlayerInformation> {
        let conn = Connection::open(db_path)?;
        let mut stmt = conn.prepare("SELECT * FROM info LIMIT 1")?;
        let info = stmt.query_row([], PlayerInformation::from_row)?;
        Ok(info)
    }

    /// Create a new rival database with player information.
    pub fn create_rival_db(&self, info: &PlayerInformation) -> Result<PathBuf> {
        let filename = format!("{}.db", info.id);
        let db_path = self.rival_dir.join(&filename);
        let conn = Connection::open(&db_path)?;
        ensure_table(&conn, &INFO_TABLE)?;
        ensure_table(&conn, &SCORE_TABLE)?;
        conn.execute(
            "INSERT OR REPLACE INTO info (id, name, rank) VALUES (?1, ?2, ?3)",
            rusqlite::params![info.id, info.name, info.rank],
        )?;
        Ok(db_path)
    }

    /// Get rival score by SHA-256 hash.
    pub fn get_rival_score(db_path: &Path, sha256: &str, mode: i32) -> Result<Option<ScoreData>> {
        let conn = Connection::open(db_path)?;
        let mut stmt = conn.prepare("SELECT * FROM score WHERE sha256 = ?1 AND mode = ?2")?;
        let result = stmt.query_row(rusqlite::params![sha256, mode], score_data_from_row);
        match result {
            Ok(score) => Ok(Some(score)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_rule::ClearType;
    use tempfile::TempDir;

    fn sample_info() -> PlayerInformation {
        PlayerInformation {
            id: "rival001".to_string(),
            name: "RivalPlayer".to_string(),
            rank: "5dan".to_string(),
        }
    }

    #[test]
    fn create_and_load_rival() {
        let tmp = TempDir::new().unwrap();
        let mut accessor = RivalDataAccessor::new(tmp.path().join("rivals")).unwrap();

        let info = sample_info();
        let db_path = accessor.create_rival_db(&info).unwrap();
        assert!(db_path.exists());

        accessor.load_local_rivals().unwrap();
        assert_eq!(accessor.rival_count(), 1);

        let rival = accessor.get_rival(0).unwrap();
        assert_eq!(rival.info.id, "rival001");
        assert_eq!(rival.info.name, "RivalPlayer");
        assert_eq!(rival.info.rank, "5dan");
    }

    #[test]
    fn load_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let mut accessor = RivalDataAccessor::new(tmp.path().join("rivals")).unwrap();
        accessor.load_local_rivals().unwrap();
        assert_eq!(accessor.rival_count(), 0);
    }

    #[test]
    fn get_rival_score_found() {
        let tmp = TempDir::new().unwrap();
        let accessor = RivalDataAccessor::new(tmp.path().join("rivals")).unwrap();
        let info = sample_info();
        let db_path = accessor.create_rival_db(&info).unwrap();

        // Insert a score
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "INSERT INTO score (sha256, mode, clear, epg, lpg, egr, lgr, egd, lgd, ebd, lbd, epr, lpr, ems, lms, notes, combo, minbp, avgjudge, playcount, clearcount, date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
            rusqlite::params!["abc123", 7, ClearType::Hard.id(), 100, 50, 30, 20, 5, 3, 1, 1, 0, 0, 0, 0, 210, 200, 5, i64::MAX, 10, 5, 1700000000i64],
        ).unwrap();

        let score = RivalDataAccessor::get_rival_score(&db_path, "abc123", 7)
            .unwrap()
            .unwrap();
        assert_eq!(score.epg, 100);
        assert_eq!(score.lpg, 50);
        assert_eq!(score.clear, ClearType::Hard);
    }

    #[test]
    fn get_rival_score_not_found() {
        let tmp = TempDir::new().unwrap();
        let accessor = RivalDataAccessor::new(tmp.path().join("rivals")).unwrap();
        let info = sample_info();
        let db_path = accessor.create_rival_db(&info).unwrap();

        let result = RivalDataAccessor::get_rival_score(&db_path, "nonexistent", 7).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn multiple_rivals() {
        let tmp = TempDir::new().unwrap();
        let mut accessor = RivalDataAccessor::new(tmp.path().join("rivals")).unwrap();

        for i in 0..3 {
            let info = PlayerInformation {
                id: format!("rival{i:03}"),
                name: format!("Player{i}"),
                rank: String::new(),
            };
            accessor.create_rival_db(&info).unwrap();
        }

        accessor.load_local_rivals().unwrap();
        assert_eq!(accessor.rival_count(), 3);
    }
}

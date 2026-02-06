use anyhow::Result;
use rusqlite::{Connection, params};

use super::models::ScoreData;

/// Score database accessor using SQLite.
/// Compatible with beatoraja's score.db schema.
pub struct ScoreDatabase {
    conn: Connection,
}

impl ScoreDatabase {
    /// Open or create a score database at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA synchronous = OFF; PRAGMA journal_mode = WAL;")?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS score (
                sha256 TEXT NOT NULL,
                mode INTEGER NOT NULL DEFAULT 0,
                clear INTEGER NOT NULL DEFAULT 0,
                epg INTEGER NOT NULL DEFAULT 0,
                lpg INTEGER NOT NULL DEFAULT 0,
                egr INTEGER NOT NULL DEFAULT 0,
                lgr INTEGER NOT NULL DEFAULT 0,
                egd INTEGER NOT NULL DEFAULT 0,
                lgd INTEGER NOT NULL DEFAULT 0,
                ebd INTEGER NOT NULL DEFAULT 0,
                lbd INTEGER NOT NULL DEFAULT 0,
                epr INTEGER NOT NULL DEFAULT 0,
                lpr INTEGER NOT NULL DEFAULT 0,
                ems INTEGER NOT NULL DEFAULT 0,
                lms INTEGER NOT NULL DEFAULT 0,
                notes INTEGER NOT NULL DEFAULT 0,
                combo INTEGER NOT NULL DEFAULT 0,
                minbp INTEGER NOT NULL DEFAULT 2147483647,
                avgjudge INTEGER NOT NULL DEFAULT 2147483647,
                playcount INTEGER NOT NULL DEFAULT 0,
                clearcount INTEGER NOT NULL DEFAULT 0,
                trophy TEXT NOT NULL DEFAULT '',
                ghost TEXT NOT NULL DEFAULT '',
                option INTEGER NOT NULL DEFAULT 0,
                seed INTEGER NOT NULL DEFAULT 0,
                random INTEGER NOT NULL DEFAULT 0,
                date INTEGER NOT NULL DEFAULT 0,
                state INTEGER NOT NULL DEFAULT 0,
                scorehash TEXT NOT NULL DEFAULT '',
                UNIQUE(sha256, mode)
            );
            CREATE INDEX IF NOT EXISTS idx_score_sha256 ON score(sha256);

            CREATE TABLE IF NOT EXISTS player (
                date INTEGER NOT NULL DEFAULT 0,
                playcount INTEGER NOT NULL DEFAULT 0,
                clear INTEGER NOT NULL DEFAULT 0,
                epg INTEGER NOT NULL DEFAULT 0,
                lpg INTEGER NOT NULL DEFAULT 0,
                egr INTEGER NOT NULL DEFAULT 0,
                lgr INTEGER NOT NULL DEFAULT 0,
                egd INTEGER NOT NULL DEFAULT 0,
                lgd INTEGER NOT NULL DEFAULT 0,
                ebd INTEGER NOT NULL DEFAULT 0,
                lbd INTEGER NOT NULL DEFAULT 0,
                epr INTEGER NOT NULL DEFAULT 0,
                lpr INTEGER NOT NULL DEFAULT 0,
                ems INTEGER NOT NULL DEFAULT 0,
                lms INTEGER NOT NULL DEFAULT 0,
                playtime INTEGER NOT NULL DEFAULT 0,
                maxcombo INTEGER NOT NULL DEFAULT 0,
                UNIQUE(date)
            );

            CREATE TABLE IF NOT EXISTS info (
                id TEXT NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                rank TEXT NOT NULL DEFAULT '',
                UNIQUE(id)
            );",
        )?;
        Ok(())
    }

    /// Get score for a specific chart and mode.
    pub fn get_score(&self, sha256: &str, mode: i32) -> Result<Option<ScoreData>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM score WHERE sha256 = ?1 AND mode = ?2 ORDER BY clear DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![sha256, mode], ScoreData::from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Get scores for multiple charts at once.
    pub fn get_scores_by_hashes(&self, sha256_list: &[&str], mode: i32) -> Result<Vec<ScoreData>> {
        if sha256_list.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = (0..sha256_list.len())
            .map(|i| format!("?{}", i + 1))
            .collect();
        let sql = format!(
            "SELECT * FROM score WHERE sha256 IN ({}) AND mode = ?{}",
            placeholders.join(","),
            sha256_list.len() + 1
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut sql_params: Vec<&dyn rusqlite::types::ToSql> = sha256_list
            .iter()
            .map(|h| h as &dyn rusqlite::types::ToSql)
            .collect();
        sql_params.push(&mode);
        let rows = stmt.query_map(sql_params.as_slice(), ScoreData::from_row)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Insert or replace a score record.
    pub fn upsert_score(&self, score: &ScoreData) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO score
                (sha256, mode, clear, epg, lpg, egr, lgr, egd, lgd, ebd, lbd, epr, lpr,
                 ems, lms, notes, combo, minbp, avgjudge, playcount, clearcount,
                 trophy, ghost, option, seed, random, date, state, scorehash)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29)",
            params![
                score.sha256,
                score.mode,
                score.clear,
                score.epg,
                score.lpg,
                score.egr,
                score.lgr,
                score.egd,
                score.lgd,
                score.ebd,
                score.lbd,
                score.epr,
                score.lpr,
                score.ems,
                score.lms,
                score.notes,
                score.combo,
                score.minbp,
                score.avgjudge,
                score.playcount,
                score.clearcount,
                score.trophy,
                score.ghost,
                score.option,
                score.seed,
                score.random,
                score.date,
                score.state,
                score.scorehash,
            ],
        )?;
        Ok(())
    }

    /// Delete a score by sha256 and mode.
    pub fn delete_score(&self, sha256: &str, mode: i32) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM score WHERE sha256 = ?1 AND mode = ?2",
            params![sha256, mode],
        )?;
        Ok(count)
    }

    /// Get total score count.
    pub fn score_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM score", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_score(sha256: &str, mode: i32, clear: i32, epg: i32, egr: i32) -> ScoreData {
        ScoreData {
            sha256: sha256.to_string(),
            mode,
            clear,
            epg,
            egr,
            notes: 100,
            ..Default::default()
        }
    }

    #[test]
    fn create_and_query_score() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        let score = make_score("chart_hash", 0, 5, 80, 20);
        db.upsert_score(&score).unwrap();

        let result = db.get_score("chart_hash", 0).unwrap();
        assert!(result.is_some());
        let s = result.unwrap();
        assert_eq!(s.clear, 5);
        assert_eq!(s.epg, 80);
        assert_eq!(s.egr, 20);
    }

    #[test]
    fn upsert_replaces_by_sha256_and_mode() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        db.upsert_score(&make_score("hash1", 0, 3, 50, 30)).unwrap();
        db.upsert_score(&make_score("hash1", 0, 7, 90, 10)).unwrap();

        assert_eq!(db.score_count().unwrap(), 1);
        let s = db.get_score("hash1", 0).unwrap().unwrap();
        assert_eq!(s.clear, 7);
    }

    #[test]
    fn different_modes_are_separate() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        db.upsert_score(&make_score("hash1", 0, 5, 80, 20)).unwrap();
        db.upsert_score(&make_score("hash1", 1, 3, 60, 30)).unwrap();

        assert_eq!(db.score_count().unwrap(), 2);
        assert_eq!(db.get_score("hash1", 0).unwrap().unwrap().clear, 5);
        assert_eq!(db.get_score("hash1", 1).unwrap().unwrap().clear, 3);
    }

    #[test]
    fn get_scores_by_hashes() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        db.upsert_score(&make_score("h1", 0, 5, 80, 20)).unwrap();
        db.upsert_score(&make_score("h2", 0, 3, 60, 30)).unwrap();
        db.upsert_score(&make_score("h3", 0, 7, 90, 10)).unwrap();

        let results = db.get_scores_by_hashes(&["h1", "h3"], 0).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn delete_score() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        db.upsert_score(&make_score("del_hash", 0, 5, 80, 20))
            .unwrap();
        assert_eq!(db.score_count().unwrap(), 1);

        db.delete_score("del_hash", 0).unwrap();
        assert_eq!(db.score_count().unwrap(), 0);
    }

    #[test]
    fn no_score_returns_none() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        let result = db.get_score("nonexistent", 0).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn score_update_best_values() {
        let mut old = make_score("h", 0, 3, 50, 30);
        let new_score = make_score("h", 0, 7, 90, 10);
        let updated = old.update(&new_score);
        assert!(updated);
        assert_eq!(old.clear, 7);
        assert_eq!(old.epg, 90);
    }
}

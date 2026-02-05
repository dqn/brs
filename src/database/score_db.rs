use anyhow::{Context, Result};
use rusqlite::params;

use crate::database::connection::Database;
use crate::database::models::{ClearType, ScoreData};

/// Accessor for score database operations.
pub struct ScoreDatabaseAccessor<'a> {
    db: &'a Database,
}

impl<'a> ScoreDatabaseAccessor<'a> {
    /// Create a new accessor for the given database.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Save a score to the database.
    /// Updates best records: clear type, ex_score, max_combo, min_bp.
    /// Always increments play_count.
    pub fn save_score(&self, score: &ScoreData) -> Result<()> {
        let conn = self.db.connection();

        // Get existing score if any
        let existing = self.get_score(&score.sha256, score.mode)?;

        let (clear, ex_score, max_combo, min_bp, clear_count) = match existing {
            Some(ref old) => {
                // Update best records
                let clear = if score.clear > old.clear {
                    score.clear
                } else {
                    old.clear
                };
                let ex_score = score.ex_score.max(old.ex_score);
                let max_combo = score.max_combo.max(old.max_combo);
                let min_bp = score.bp().min(old.min_bp);
                let clear_count = if score.clear.is_cleared() {
                    old.clear_count + 1
                } else {
                    old.clear_count
                };
                (clear, ex_score, max_combo, min_bp, clear_count)
            }
            None => {
                let clear_count = if score.clear.is_cleared() { 1 } else { 0 };
                (
                    score.clear,
                    score.ex_score,
                    score.max_combo,
                    score.bp(),
                    clear_count,
                )
            }
        };

        let play_count = existing.as_ref().map_or(1, |old| old.play_count + 1);

        conn.execute(
            r#"
            INSERT INTO score (
                sha256, mode, clear, ex_score, max_combo, min_bp,
                pg, gr, gd, bd, pr, ms, notes, play_count, clear_count, date
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
            )
            ON CONFLICT(sha256, mode) DO UPDATE SET
                clear = excluded.clear,
                ex_score = excluded.ex_score,
                max_combo = excluded.max_combo,
                min_bp = excluded.min_bp,
                pg = excluded.pg,
                gr = excluded.gr,
                gd = excluded.gd,
                bd = excluded.bd,
                pr = excluded.pr,
                ms = excluded.ms,
                notes = excluded.notes,
                play_count = excluded.play_count,
                clear_count = excluded.clear_count,
                date = excluded.date
            "#,
            params![
                score.sha256,
                score.mode,
                clear.as_i32(),
                ex_score,
                max_combo,
                min_bp,
                score.pg,
                score.gr,
                score.gd,
                score.bd,
                score.pr,
                score.ms,
                score.notes,
                play_count,
                clear_count,
                score.date,
            ],
        )
        .context("Failed to save score")?;

        Ok(())
    }

    /// Get a score by SHA256 hash and mode.
    pub fn get_score(&self, sha256: &str, mode: i32) -> Result<Option<ScoreData>> {
        let conn = self.db.connection();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT sha256, mode, clear, ex_score, max_combo, min_bp,
                       pg, gr, gd, bd, pr, ms, notes, play_count, clear_count, date
                FROM score WHERE sha256 = ?1 AND mode = ?2
                "#,
            )
            .context("Failed to prepare statement")?;

        let result = stmt.query_row([sha256, &mode.to_string()], |row| {
            Ok(ScoreData {
                sha256: row.get(0)?,
                mode: row.get(1)?,
                clear: ClearType::from_i32(row.get(2)?).unwrap_or_default(),
                ex_score: row.get(3)?,
                max_combo: row.get(4)?,
                min_bp: row.get(5)?,
                pg: row.get(6)?,
                gr: row.get(7)?,
                gd: row.get(8)?,
                bd: row.get(9)?,
                pr: row.get(10)?,
                ms: row.get(11)?,
                notes: row.get(12)?,
                play_count: row.get(13)?,
                clear_count: row.get(14)?,
                date: row.get(15)?,
            })
        });

        match result {
            Ok(score) => Ok(Some(score)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query score"),
        }
    }

    /// Get all scores from the database.
    pub fn get_all_scores(&self) -> Result<Vec<ScoreData>> {
        let conn = self.db.connection();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT sha256, mode, clear, ex_score, max_combo, min_bp,
                       pg, gr, gd, bd, pr, ms, notes, play_count, clear_count, date
                FROM score ORDER BY date DESC
                "#,
            )
            .context("Failed to prepare statement")?;

        let scores = stmt
            .query_map([], |row| {
                Ok(ScoreData {
                    sha256: row.get(0)?,
                    mode: row.get(1)?,
                    clear: ClearType::from_i32(row.get(2)?).unwrap_or_default(),
                    ex_score: row.get(3)?,
                    max_combo: row.get(4)?,
                    min_bp: row.get(5)?,
                    pg: row.get(6)?,
                    gr: row.get(7)?,
                    gd: row.get(8)?,
                    bd: row.get(9)?,
                    pr: row.get(10)?,
                    ms: row.get(11)?,
                    notes: row.get(12)?,
                    play_count: row.get(13)?,
                    clear_count: row.get(14)?,
                    date: row.get(15)?,
                })
            })
            .context("Failed to query all scores")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect scores")?;

        Ok(scores)
    }

    /// Get the number of scores in the database.
    pub fn count(&self) -> Result<usize> {
        let conn = self.db.connection();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM score", [], |row| row.get(0))
            .context("Failed to count scores")?;

        Ok(count as usize)
    }

    /// Delete a score by SHA256 hash and mode.
    pub fn delete_score(&self, sha256: &str, mode: i32) -> Result<bool> {
        let conn = self.db.connection();

        let rows_affected = conn
            .execute(
                "DELETE FROM score WHERE sha256 = ?1 AND mode = ?2",
                params![sha256, mode],
            )
            .context("Failed to delete score")?;

        Ok(rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::Database;

    fn create_test_score(sha256: &str, clear: ClearType, ex_score: i32) -> ScoreData {
        let mut score = ScoreData::new(sha256.to_string());
        score.clear = clear;
        score.ex_score = ex_score;
        score.max_combo = 100;
        score.pg = ex_score / 2;
        score.gr = ex_score % 2;
        score.notes = 500;
        score.date = 1234567890;
        score
    }

    #[test]
    fn test_save_and_get_score() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        let score = create_test_score("abc123", ClearType::Normal, 800);
        accessor.save_score(&score).unwrap();

        let retrieved = accessor.get_score("abc123", 0).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.clear, ClearType::Normal);
        assert_eq!(retrieved.ex_score, 800);
    }

    #[test]
    fn test_best_clear_update() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        // First play: Normal clear
        let score1 = create_test_score("abc123", ClearType::Normal, 800);
        accessor.save_score(&score1).unwrap();

        // Second play: Hard clear (better)
        let score2 = create_test_score("abc123", ClearType::Hard, 750);
        accessor.save_score(&score2).unwrap();

        let retrieved = accessor.get_score("abc123", 0).unwrap().unwrap();
        assert_eq!(retrieved.clear, ClearType::Hard);
        assert_eq!(retrieved.play_count, 2);
    }

    #[test]
    fn test_best_clear_not_downgrade() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        // First play: Hard clear
        let score1 = create_test_score("abc123", ClearType::Hard, 800);
        accessor.save_score(&score1).unwrap();

        // Second play: Normal clear (worse)
        let score2 = create_test_score("abc123", ClearType::Normal, 850);
        accessor.save_score(&score2).unwrap();

        let retrieved = accessor.get_score("abc123", 0).unwrap().unwrap();
        // Clear should remain Hard (the better one)
        assert_eq!(retrieved.clear, ClearType::Hard);
        // But ex_score should be updated to the higher value
        assert_eq!(retrieved.ex_score, 850);
    }

    #[test]
    fn test_best_ex_score_update() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        let score1 = create_test_score("abc123", ClearType::Normal, 800);
        accessor.save_score(&score1).unwrap();

        let score2 = create_test_score("abc123", ClearType::Normal, 900);
        accessor.save_score(&score2).unwrap();

        let retrieved = accessor.get_score("abc123", 0).unwrap().unwrap();
        assert_eq!(retrieved.ex_score, 900);
    }

    #[test]
    fn test_play_count_increment() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        let score = create_test_score("abc123", ClearType::Normal, 800);

        accessor.save_score(&score).unwrap();
        assert_eq!(
            accessor.get_score("abc123", 0).unwrap().unwrap().play_count,
            1
        );

        accessor.save_score(&score).unwrap();
        assert_eq!(
            accessor.get_score("abc123", 0).unwrap().unwrap().play_count,
            2
        );

        accessor.save_score(&score).unwrap();
        assert_eq!(
            accessor.get_score("abc123", 0).unwrap().unwrap().play_count,
            3
        );
    }

    #[test]
    fn test_clear_count_increment() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        // Failed play
        let score1 = create_test_score("abc123", ClearType::Failed, 400);
        accessor.save_score(&score1).unwrap();
        assert_eq!(
            accessor
                .get_score("abc123", 0)
                .unwrap()
                .unwrap()
                .clear_count,
            0
        );

        // Clear play
        let score2 = create_test_score("abc123", ClearType::Normal, 800);
        accessor.save_score(&score2).unwrap();
        assert_eq!(
            accessor
                .get_score("abc123", 0)
                .unwrap()
                .unwrap()
                .clear_count,
            1
        );

        // Another clear play
        accessor.save_score(&score2).unwrap();
        assert_eq!(
            accessor
                .get_score("abc123", 0)
                .unwrap()
                .unwrap()
                .clear_count,
            2
        );
    }

    #[test]
    fn test_get_all_scores() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        accessor
            .save_score(&create_test_score("abc", ClearType::Normal, 800))
            .unwrap();
        accessor
            .save_score(&create_test_score("def", ClearType::Hard, 900))
            .unwrap();
        accessor
            .save_score(&create_test_score("ghi", ClearType::Easy, 700))
            .unwrap();

        let scores = accessor.get_all_scores().unwrap();
        assert_eq!(scores.len(), 3);
    }

    #[test]
    fn test_delete_score() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        let score = create_test_score("abc123", ClearType::Normal, 800);
        accessor.save_score(&score).unwrap();

        assert!(accessor.delete_score("abc123", 0).unwrap());
        assert!(accessor.get_score("abc123", 0).unwrap().is_none());
    }

    #[test]
    fn test_min_bp_update() {
        let db = Database::open_score_db_in_memory().unwrap();
        let accessor = ScoreDatabaseAccessor::new(&db);

        let mut score1 = create_test_score("abc123", ClearType::Normal, 800);
        score1.bd = 5;
        score1.pr = 3;
        score1.ms = 2; // BP = 10
        accessor.save_score(&score1).unwrap();

        let mut score2 = create_test_score("abc123", ClearType::Normal, 800);
        score2.bd = 2;
        score2.pr = 1;
        score2.ms = 1; // BP = 4
        accessor.save_score(&score2).unwrap();

        let retrieved = accessor.get_score("abc123", 0).unwrap().unwrap();
        assert_eq!(retrieved.min_bp, 4);
    }
}

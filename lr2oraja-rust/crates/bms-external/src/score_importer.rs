use std::path::Path;

use anyhow::{Result, anyhow};
use bms_rule::{ClearType, ScoreData};
use rusqlite::Connection;
use tracing::{info, warn};

/// LR2 clear value to beatoraja ClearType mapping.
///
/// LR2 clear values: [0, 1, 4, 5, 6, 8, 9]
/// Mapped to: [NoPlay, Failed, Easy, Normal, Hard, FullCombo, Perfect]
fn lr2_clear_to_clear_type(lr2_clear: i32) -> ClearType {
    match lr2_clear {
        0 => ClearType::NoPlay,
        1 => ClearType::Failed,
        4 => ClearType::Easy,
        5 => ClearType::Normal,
        6 => ClearType::Hard,
        8 => ClearType::FullCombo,
        9 => ClearType::Perfect,
        // LR2 values 2, 3 are assist/light assist but not standard
        2 => ClearType::AssistEasy,
        3 => ClearType::LightAssistEasy,
        7 => ClearType::ExHard,
        _ => ClearType::NoPlay,
    }
}

/// Result of a score import operation.
#[derive(Debug, Clone, Default)]
pub struct ImportResult {
    pub imported_count: u32,
    pub skipped_count: u32,
    pub error_count: u32,
}

/// Score data importer from LR2 database format.
pub struct ScoreDataImporter;

impl ScoreDataImporter {
    /// Import scores from an LR2 SQLite database into the bms score database.
    ///
    /// Opens the LR2 DB read-only, reads the score table, converts clear values,
    /// and writes to the target ScoreDatabase.
    pub fn import_from_lr2(
        lr2_db_path: impl AsRef<Path>,
        score_db: &bms_database::ScoreDatabase,
    ) -> Result<ImportResult> {
        let lr2_path = lr2_db_path.as_ref();
        if !lr2_path.exists() {
            return Err(anyhow!("LR2 database not found: {}", lr2_path.display()));
        }

        let lr2_conn = Connection::open_with_flags(
            lr2_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let mut result = ImportResult::default();

        // LR2 score table structure: hash (md5), clear, perfect, great, good, bad, poor, combo, minbp, playcount, clearcount, option
        let mut stmt = lr2_conn.prepare(
            "SELECT hash, clear, perfect, great, good, bad, poor, maxcombo, minbp, playcount, clearcount FROM score",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Lr2ScoreRow {
                hash: row.get(0)?,
                clear: row.get(1)?,
                perfect: row.get(2)?,
                great: row.get(3)?,
                good: row.get(4)?,
                bad: row.get(5)?,
                poor: row.get(6)?,
                maxcombo: row.get(7)?,
                minbp: row.get(8)?,
                playcount: row.get(9)?,
                clearcount: row.get(10)?,
            })
        })?;

        for row_result in rows {
            match row_result {
                Ok(lr2_row) => {
                    if lr2_row.hash.is_empty() {
                        result.skipped_count += 1;
                        continue;
                    }

                    let score = convert_lr2_score(&lr2_row);
                    match score_db.set_score_data(&[score]) {
                        Ok(()) => result.imported_count += 1,
                        Err(e) => {
                            warn!("failed to import score for {}: {}", lr2_row.hash, e);
                            result.error_count += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("failed to read LR2 score row: {}", e);
                    result.error_count += 1;
                }
            }
        }

        info!(
            "LR2 import complete: {} imported, {} skipped, {} errors",
            result.imported_count, result.skipped_count, result.error_count
        );

        Ok(result)
    }
}

/// Internal representation of an LR2 score row.
struct Lr2ScoreRow {
    hash: String,
    clear: i32,
    perfect: i32,
    great: i32,
    good: i32,
    bad: i32,
    poor: i32,
    maxcombo: i32,
    minbp: i32,
    playcount: i32,
    clearcount: i32,
}

/// Convert an LR2 score row to a ScoreData.
///
/// LR2 does not distinguish early/late, so all counts go to the "early" fields.
/// LR2 uses MD5 for hash (stored in sha256 field for lookup purposes).
fn convert_lr2_score(lr2: &Lr2ScoreRow) -> ScoreData {
    ScoreData {
        sha256: lr2.hash.clone(), // Actually MD5 from LR2
        clear: lr2_clear_to_clear_type(lr2.clear),
        epg: lr2.perfect,
        egr: lr2.great,
        egd: lr2.good,
        ebd: lr2.bad,
        epr: lr2.poor,
        maxcombo: lr2.maxcombo,
        minbp: lr2.minbp,
        playcount: lr2.playcount,
        clearcount: lr2.clearcount,
        notes: lr2.perfect + lr2.great + lr2.good + lr2.bad + lr2.poor,
        ..ScoreData::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lr2_clear_mapping() {
        assert_eq!(lr2_clear_to_clear_type(0), ClearType::NoPlay);
        assert_eq!(lr2_clear_to_clear_type(1), ClearType::Failed);
        assert_eq!(lr2_clear_to_clear_type(2), ClearType::AssistEasy);
        assert_eq!(lr2_clear_to_clear_type(3), ClearType::LightAssistEasy);
        assert_eq!(lr2_clear_to_clear_type(4), ClearType::Easy);
        assert_eq!(lr2_clear_to_clear_type(5), ClearType::Normal);
        assert_eq!(lr2_clear_to_clear_type(6), ClearType::Hard);
        assert_eq!(lr2_clear_to_clear_type(7), ClearType::ExHard);
        assert_eq!(lr2_clear_to_clear_type(8), ClearType::FullCombo);
        assert_eq!(lr2_clear_to_clear_type(9), ClearType::Perfect);
        assert_eq!(lr2_clear_to_clear_type(99), ClearType::NoPlay);
        assert_eq!(lr2_clear_to_clear_type(-1), ClearType::NoPlay);
    }

    #[test]
    fn convert_lr2_score_basic() {
        let row = Lr2ScoreRow {
            hash: "abc123def456".to_string(),
            clear: 5,
            perfect: 100,
            great: 50,
            good: 10,
            bad: 5,
            poor: 3,
            maxcombo: 80,
            minbp: 8,
            playcount: 10,
            clearcount: 5,
        };
        let score = convert_lr2_score(&row);
        assert_eq!(score.sha256, "abc123def456");
        assert_eq!(score.clear, ClearType::Normal);
        assert_eq!(score.epg, 100);
        assert_eq!(score.egr, 50);
        assert_eq!(score.egd, 10);
        assert_eq!(score.ebd, 5);
        assert_eq!(score.epr, 3);
        assert_eq!(score.maxcombo, 80);
        assert_eq!(score.minbp, 8);
        assert_eq!(score.playcount, 10);
        assert_eq!(score.clearcount, 5);
        assert_eq!(score.notes, 168); // 100+50+10+5+3
    }

    #[test]
    fn convert_lr2_score_late_fields_are_zero() {
        let row = Lr2ScoreRow {
            hash: "test".to_string(),
            clear: 1,
            perfect: 50,
            great: 30,
            good: 20,
            bad: 10,
            poor: 5,
            maxcombo: 40,
            minbp: 15,
            playcount: 1,
            clearcount: 0,
        };
        let score = convert_lr2_score(&row);
        assert_eq!(score.lpg, 0);
        assert_eq!(score.lgr, 0);
        assert_eq!(score.lgd, 0);
        assert_eq!(score.lbd, 0);
        assert_eq!(score.lpr, 0);
        assert_eq!(score.lms, 0);
        assert_eq!(score.ems, 0);
    }

    #[test]
    fn import_result_default() {
        let result = ImportResult::default();
        assert_eq!(result.imported_count, 0);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn import_from_nonexistent_path() {
        let score_db = bms_database::ScoreDatabase::open_in_memory().unwrap();
        let result = ScoreDataImporter::import_from_lr2("/nonexistent/path/lr2.db", &score_db);
        assert!(result.is_err());
    }

    #[test]
    fn import_from_lr2_with_mock_db() {
        let dir = tempfile::tempdir().unwrap();
        let lr2_path = dir.path().join("lr2score.db");

        // Create a minimal LR2-style score DB
        let lr2_conn = Connection::open(&lr2_path).unwrap();
        lr2_conn
            .execute_batch(
                "CREATE TABLE score (
                    hash TEXT PRIMARY KEY,
                    clear INTEGER DEFAULT 0,
                    perfect INTEGER DEFAULT 0,
                    great INTEGER DEFAULT 0,
                    good INTEGER DEFAULT 0,
                    bad INTEGER DEFAULT 0,
                    poor INTEGER DEFAULT 0,
                    maxcombo INTEGER DEFAULT 0,
                    minbp INTEGER DEFAULT 0,
                    playcount INTEGER DEFAULT 0,
                    clearcount INTEGER DEFAULT 0
                );
                INSERT INTO score VALUES ('md5hash1', 5, 100, 50, 10, 5, 3, 80, 8, 10, 5);
                INSERT INTO score VALUES ('md5hash2', 8, 200, 30, 5, 0, 0, 235, 0, 20, 20);
                INSERT INTO score VALUES ('', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);",
            )
            .unwrap();
        drop(lr2_conn);

        let score_db = bms_database::ScoreDatabase::open_in_memory().unwrap();
        let result = ScoreDataImporter::import_from_lr2(&lr2_path, &score_db).unwrap();

        assert_eq!(result.imported_count, 2);
        assert_eq!(result.skipped_count, 1); // empty hash
        assert_eq!(result.error_count, 0);
    }
}

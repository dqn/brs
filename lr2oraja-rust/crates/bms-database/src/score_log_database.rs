use std::path::Path;

use anyhow::Result;
use bms_rule::ScoreData;
use rusqlite::Connection;

use crate::schema::{SCOREDATALOG_TABLE, ensure_table};

/// Score data log database accessor (scoredatalog.db).
///
/// Append-only log of score records for play history.
pub struct ScoreDataLogDatabase {
    conn: Connection,
}

impl ScoreDataLogDatabase {
    /// Open (or create) a score data log database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA synchronous = OFF;")?;
        ensure_table(&conn, &SCOREDATALOG_TABLE)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        ensure_table(&conn, &SCOREDATALOG_TABLE)?;
        Ok(Self { conn })
    }

    /// Append score data logs (INSERT OR REPLACE).
    pub fn set_score_data_log(&self, scores: &[ScoreData]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO scoredatalog \
                 (sha256,mode,clear,epg,lpg,egr,lgr,egd,lgd,ebd,lbd,epr,lpr,ems,lms,\
                  notes,combo,minbp,avgjudge,playcount,clearcount,trophy,ghost,\
                  [option],seed,random,date,state,scorehash) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,\
                         ?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29)",
            )?;
            for sd in scores {
                stmt.execute(rusqlite::params![
                    sd.sha256,
                    sd.mode,
                    sd.clear.id(),
                    sd.epg,
                    sd.lpg,
                    sd.egr,
                    sd.lgr,
                    sd.egd,
                    sd.lgd,
                    sd.ebd,
                    sd.lbd,
                    sd.epr,
                    sd.lpr,
                    sd.ems,
                    sd.lms,
                    sd.notes,
                    sd.maxcombo,
                    sd.minbp,
                    sd.avgjudge,
                    sd.playcount,
                    sd.clearcount,
                    sd.trophy,
                    sd.ghost,
                    sd.option,
                    sd.seed,
                    sd.random,
                    sd.date,
                    sd.state,
                    sd.scorehash,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_rule::ClearType;

    #[test]
    fn score_log_round_trip() {
        let db = ScoreDataLogDatabase::open_in_memory().unwrap();
        let score = ScoreData {
            sha256: "test_hash".to_string(),
            mode: 7,
            clear: ClearType::Normal,
            epg: 50,
            notes: 200,
            ..Default::default()
        };

        db.set_score_data_log(&[score]).unwrap();

        // Verify by reading back
        let mut stmt = db
            .conn
            .prepare("SELECT * FROM scoredatalog WHERE sha256 = 'test_hash'")
            .unwrap();
        let count: i32 = db
            .conn
            .query_row("SELECT COUNT(*) FROM scoredatalog", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let rows: Vec<_> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>("sha256")?,
                    row.get::<_, i32>("epg")?,
                    row.get::<_, i32>("clear")?,
                ))
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "test_hash");
        assert_eq!(rows[0].1, 50);
        assert_eq!(rows[0].2, ClearType::Normal.id() as i32);
    }
}

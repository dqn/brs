use std::path::Path;

use anyhow::Result;
use bms_rule::{ClearType, ScoreData};
use rusqlite::Connection;

use crate::player_data::PlayerData;
use crate::player_info::PlayerInformation;
use crate::schema::{INFO_TABLE, PLAYER_TABLE, SCORE_TABLE, ensure_table};

const LOAD_CHUNK_SIZE: usize = 1000;

/// Score database accessor (score.db).
///
/// Manages the `info`, `player`, and `score` tables.
pub struct ScoreDatabase {
    conn: Connection,
}

/// Read a ScoreData from a rusqlite row.
///
/// Note: DB column `combo` maps to ScoreData field `maxcombo`.
pub(crate) fn score_data_from_row(row: &rusqlite::Row) -> rusqlite::Result<ScoreData> {
    let clear_id: i32 = row.get::<_, Option<i32>>("clear")?.unwrap_or(0);
    Ok(ScoreData {
        sha256: row.get("sha256")?,
        player: String::new(),
        mode: row.get::<_, Option<i32>>("mode")?.unwrap_or(0),
        clear: ClearType::from_id(clear_id as u8).unwrap_or_default(),
        date: row.get::<_, Option<i64>>("date")?.unwrap_or(0),
        playcount: row.get::<_, Option<i32>>("playcount")?.unwrap_or(0),
        clearcount: row.get::<_, Option<i32>>("clearcount")?.unwrap_or(0),
        epg: row.get::<_, Option<i32>>("epg")?.unwrap_or(0),
        lpg: row.get::<_, Option<i32>>("lpg")?.unwrap_or(0),
        egr: row.get::<_, Option<i32>>("egr")?.unwrap_or(0),
        lgr: row.get::<_, Option<i32>>("lgr")?.unwrap_or(0),
        egd: row.get::<_, Option<i32>>("egd")?.unwrap_or(0),
        lgd: row.get::<_, Option<i32>>("lgd")?.unwrap_or(0),
        ebd: row.get::<_, Option<i32>>("ebd")?.unwrap_or(0),
        lbd: row.get::<_, Option<i32>>("lbd")?.unwrap_or(0),
        epr: row.get::<_, Option<i32>>("epr")?.unwrap_or(0),
        lpr: row.get::<_, Option<i32>>("lpr")?.unwrap_or(0),
        ems: row.get::<_, Option<i32>>("ems")?.unwrap_or(0),
        lms: row.get::<_, Option<i32>>("lms")?.unwrap_or(0),
        maxcombo: row.get::<_, Option<i32>>("combo")?.unwrap_or(0),
        notes: row.get::<_, Option<i32>>("notes")?.unwrap_or(0),
        passnotes: 0,
        minbp: row.get::<_, Option<i32>>("minbp")?.unwrap_or(i32::MAX),
        avgjudge: row.get::<_, Option<i64>>("avgjudge")?.unwrap_or(i64::MAX),
        trophy: row.get::<_, Option<String>>("trophy")?.unwrap_or_default(),
        ghost: row.get::<_, Option<String>>("ghost")?.unwrap_or_default(),
        random: row.get::<_, Option<i32>>("random")?.unwrap_or(0),
        option: row.get::<_, Option<i32>>("option")?.unwrap_or(0),
        seed: row.get::<_, Option<i64>>("seed")?.unwrap_or(-1),
        assist: 0,
        gauge: 0,
        state: row.get::<_, Option<i32>>("state")?.unwrap_or(0),
        scorehash: row
            .get::<_, Option<String>>("scorehash")?
            .unwrap_or_default(),
    })
}

impl ScoreDatabase {
    /// Open (or create) a score database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA synchronous = OFF;")?;
        ensure_table(&conn, &INFO_TABLE)?;
        ensure_table(&conn, &PLAYER_TABLE)?;
        ensure_table(&conn, &SCORE_TABLE)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        ensure_table(&conn, &INFO_TABLE)?;
        ensure_table(&conn, &PLAYER_TABLE)?;
        ensure_table(&conn, &SCORE_TABLE)?;
        Ok(Self { conn })
    }

    /// Get player information.
    pub fn get_information(&self) -> Result<Option<PlayerInformation>> {
        let mut stmt = self.conn.prepare("SELECT * FROM info")?;
        let mut rows = stmt.query_map([], PlayerInformation::from_row)?;
        match rows.next() {
            Some(Ok(info)) => Ok(Some(info)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Set player information (replaces existing).
    pub fn set_information(&self, info: &PlayerInformation) -> Result<()> {
        self.conn.execute("DELETE FROM info", [])?;
        self.conn.execute(
            "INSERT INTO info (id, name, rank) VALUES (?1, ?2, ?3)",
            rusqlite::params![info.id, info.name, info.rank],
        )?;
        Ok(())
    }

    /// Get the best score for a specific chart+mode.
    pub fn get_score_data(&self, sha256: &str, mode: i32) -> Result<Option<ScoreData>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM score WHERE sha256 = ?1 AND mode = ?2")?;
        let rows = stmt.query_map(rusqlite::params![sha256, mode], score_data_from_row)?;

        let mut best: Option<ScoreData> = None;
        for r in rows {
            let sd = r?;
            if best.as_ref().is_none_or(|b| sd.clear > b.clear) {
                best = Some(sd);
            }
        }
        Ok(best)
    }

    /// Get scores for multiple SHA256 hashes in chunks.
    pub fn get_score_datas(&self, sha256_list: &[&str], mode: i32) -> Result<Vec<ScoreData>> {
        let mut results = Vec::new();
        for chunk in sha256_list.chunks(LOAD_CHUNK_SIZE) {
            let placeholders: Vec<String> = chunk
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "SELECT * FROM score WHERE sha256 IN ({}) AND mode = ?{}",
                placeholders.join(","),
                chunk.len() + 1
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = chunk
                .iter()
                .map(|s| Box::new(s.to_string()) as Box<dyn rusqlite::types::ToSql>)
                .collect();
            params.push(Box::new(mode));
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|b| b.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), score_data_from_row)?;
            for r in rows {
                results.push(r?);
            }
        }
        Ok(results)
    }

    /// Insert or replace score data (batch, in a transaction).
    pub fn set_score_data(&self, scores: &[ScoreData]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO score \
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

    /// Delete a score entry.
    pub fn delete_score_data(&self, sha256: &str, mode: i32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM score WHERE sha256 = ?1 AND mode = ?2",
            rusqlite::params![sha256, mode],
        )?;
        Ok(())
    }

    /// Get player data records (ordered by date descending).
    pub fn get_player_datas(&self, count: i32) -> Result<Vec<PlayerData>> {
        let sql = if count > 0 {
            format!("SELECT * FROM player ORDER BY date DESC LIMIT {count}")
        } else {
            "SELECT * FROM player ORDER BY date DESC".to_string()
        };
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], PlayerData::from_row)?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// Insert or replace player data.
    pub fn set_player_data(&self, pd: &PlayerData) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO player \
             (date,playcount,clear,epg,lpg,egr,lgr,egd,lgd,ebd,lbd,epr,lpr,ems,lms,\
              playtime,maxcombo) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
            rusqlite::params![
                pd.date,
                pd.playcount,
                pd.clear,
                pd.epg,
                pd.lpg,
                pd.egr,
                pd.lgr,
                pd.egd,
                pd.lgd,
                pd.ebd,
                pd.lbd,
                pd.epr,
                pd.lpr,
                pd.ems,
                pd.lms,
                pd.playtime,
                pd.maxcombo,
            ],
        )?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_score() -> ScoreData {
        ScoreData {
            sha256: "abc123def456".to_string(),
            mode: 7,
            clear: ClearType::Hard,
            epg: 100,
            lpg: 50,
            egr: 30,
            lgr: 20,
            maxcombo: 200,
            notes: 500,
            minbp: 10,
            playcount: 5,
            clearcount: 3,
            ..Default::default()
        }
    }

    #[test]
    fn score_crud_round_trip() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        let score = sample_score();

        db.set_score_data(&[score]).unwrap();

        let found = db.get_score_data("abc123def456", 7).unwrap().unwrap();
        assert_eq!(found.clear, ClearType::Hard);
        assert_eq!(found.epg, 100);
        assert_eq!(found.lpg, 50);
        assert_eq!(found.maxcombo, 200);
        assert_eq!(found.notes, 500);
    }

    #[test]
    fn score_delete() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        db.set_score_data(&[sample_score()]).unwrap();

        db.delete_score_data("abc123def456", 7).unwrap();

        let found = db.get_score_data("abc123def456", 7).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn score_batch_get() {
        let db = ScoreDatabase::open_in_memory().unwrap();

        let mut scores = Vec::new();
        for i in 0..5 {
            let mut sd = sample_score();
            sd.sha256 = format!("hash_{i}");
            sd.epg = i;
            scores.push(sd);
        }
        db.set_score_data(&scores).unwrap();

        let hashes: Vec<&str> = (0..5).map(|i| scores[i as usize].sha256.as_str()).collect();
        let found = db.get_score_datas(&hashes, 7).unwrap();
        assert_eq!(found.len(), 5);
    }

    #[test]
    fn player_info_round_trip() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        let info = PlayerInformation {
            id: "player1".to_string(),
            name: "TestPlayer".to_string(),
            rank: "10dan".to_string(),
        };

        db.set_information(&info).unwrap();

        let found = db.get_information().unwrap().unwrap();
        assert_eq!(found.id, "player1");
        assert_eq!(found.name, "TestPlayer");
        assert_eq!(found.rank, "10dan");
    }

    #[test]
    fn player_data_round_trip() {
        let db = ScoreDatabase::open_in_memory().unwrap();
        let pd = PlayerData {
            date: 1700000000,
            playcount: 100,
            clear: 50,
            epg: 10000,
            lpg: 5000,
            maxcombo: 300,
            ..Default::default()
        };

        db.set_player_data(&pd).unwrap();

        let found = db.get_player_datas(1).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].date, 1700000000);
        assert_eq!(found[0].playcount, 100);
        assert_eq!(found[0].epg, 10000);
    }
}

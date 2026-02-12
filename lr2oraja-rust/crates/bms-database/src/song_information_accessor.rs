//! Song information database accessor.
//!
//! Port of Java `SongInformationAccessor.java`.
//! Provides CRUD operations for song information data in an SQLite database.

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use crate::schema::{INFORMATION_TABLE, ensure_table};
use crate::song_information::SongInformation;

const LOAD_CHUNK_SIZE: usize = 1000;

/// Database accessor for song information (information.db).
pub struct SongInformationAccessor {
    conn: Connection,
}

impl SongInformationAccessor {
    /// Open (or create) a song information database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA synchronous = OFF;")?;
        ensure_table(&conn, &INFORMATION_TABLE)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        ensure_table(&conn, &INFORMATION_TABLE)?;
        Ok(Self { conn })
    }

    /// Get song information by SHA-256 hash.
    pub fn get_information(&self, sha256: &str) -> Result<Option<SongInformation>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM information WHERE sha256 = ?1")?;
        let mut rows = stmt.query_map(rusqlite::params![sha256], info_from_row)?;
        match rows.next() {
            Some(Ok(info)) => {
                if info.validate() {
                    Ok(Some(info))
                } else {
                    Ok(None)
                }
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Get song information for multiple SHA-256 hashes (chunked query).
    pub fn get_informations_by_sha256s(&self, hashes: &[&str]) -> Result<Vec<SongInformation>> {
        let mut results = Vec::new();
        for chunk in hashes.chunks(LOAD_CHUNK_SIZE) {
            let placeholders: Vec<String> = chunk
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "SELECT * FROM information WHERE sha256 IN ({})",
                placeholders.join(",")
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let params: Vec<Box<dyn rusqlite::types::ToSql>> = chunk
                .iter()
                .map(|s| Box::new(s.to_string()) as Box<dyn rusqlite::types::ToSql>)
                .collect();
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params.iter().map(|b| b.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), info_from_row)?;
            for r in rows {
                let info = r?;
                if info.validate() {
                    results.push(info);
                }
            }
        }
        Ok(results)
    }

    /// Insert or replace song information (single entry).
    pub fn update(&self, info: &SongInformation) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO information \
             (sha256,n,ln,s,ls,total,density,peakdensity,enddensity,mainbpm,\
              distribution,speedchange,lanenotes) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            rusqlite::params![
                info.sha256,
                info.n,
                info.ln,
                info.s,
                info.ls,
                info.total,
                info.density,
                info.peakdensity,
                info.enddensity,
                info.mainbpm,
                info.distribution,
                info.speedchange,
                info.lanenotes,
            ],
        )?;
        Ok(())
    }

    /// Insert or replace multiple song information entries in a transaction.
    pub fn batch_update(&self, infos: &[SongInformation]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO information \
                 (sha256,n,ln,s,ls,total,density,peakdensity,enddensity,mainbpm,\
                  distribution,speedchange,lanenotes) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            )?;
            for info in infos {
                stmt.execute(rusqlite::params![
                    info.sha256,
                    info.n,
                    info.ln,
                    info.s,
                    info.ls,
                    info.total,
                    info.density,
                    info.peakdensity,
                    info.enddensity,
                    info.mainbpm,
                    info.distribution,
                    info.speedchange,
                    info.lanenotes,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

/// Read a SongInformation from a rusqlite row.
fn info_from_row(row: &rusqlite::Row) -> rusqlite::Result<SongInformation> {
    Ok(SongInformation {
        sha256: row.get("sha256")?,
        n: row.get::<_, Option<i32>>("n")?.unwrap_or(0),
        ln: row.get::<_, Option<i32>>("ln")?.unwrap_or(0),
        s: row.get::<_, Option<i32>>("s")?.unwrap_or(0),
        ls: row.get::<_, Option<i32>>("ls")?.unwrap_or(0),
        total: row.get::<_, Option<f64>>("total")?.unwrap_or(0.0),
        density: row.get::<_, Option<f64>>("density")?.unwrap_or(0.0),
        peakdensity: row.get::<_, Option<f64>>("peakdensity")?.unwrap_or(0.0),
        enddensity: row.get::<_, Option<f64>>("enddensity")?.unwrap_or(0.0),
        mainbpm: row.get::<_, Option<f64>>("mainbpm")?.unwrap_or(0.0),
        distribution: row
            .get::<_, Option<String>>("distribution")?
            .unwrap_or_default(),
        speedchange: row
            .get::<_, Option<String>>("speedchange")?
            .unwrap_or_default(),
        lanenotes: row
            .get::<_, Option<String>>("lanenotes")?
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_info() -> SongInformation {
        SongInformation {
            sha256: "a".repeat(64),
            n: 500,
            ln: 50,
            s: 30,
            ls: 10,
            total: 300.0,
            density: 8.5,
            peakdensity: 15.0,
            enddensity: 12.0,
            mainbpm: 180.0,
            distribution: "#000100".to_string(),
            speedchange: "180,0,0,5000".to_string(),
            lanenotes: "100,10,5,200,20,3".to_string(),
        }
    }

    #[test]
    fn insert_and_get() {
        let db = SongInformationAccessor::open_in_memory().unwrap();
        let info = sample_info();

        db.update(&info).unwrap();

        let found = db.get_information(&"a".repeat(64)).unwrap().unwrap();
        assert_eq!(found.sha256, "a".repeat(64));
        assert_eq!(found.n, 500);
        assert_eq!(found.ln, 50);
        assert_eq!(found.s, 30);
        assert_eq!(found.ls, 10);
        assert!((found.total - 300.0).abs() < f64::EPSILON);
        assert!((found.density - 8.5).abs() < f64::EPSILON);
        assert!((found.mainbpm - 180.0).abs() < f64::EPSILON);
    }

    #[test]
    fn get_nonexistent() {
        let db = SongInformationAccessor::open_in_memory().unwrap();
        let found = db.get_information("nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn batch_update_and_query() {
        let db = SongInformationAccessor::open_in_memory().unwrap();

        let mut infos = Vec::new();
        for i in 0..5 {
            let mut info = sample_info();
            info.sha256 = format!("{:0>64}", i);
            info.n = i * 100;
            infos.push(info);
        }

        db.batch_update(&infos).unwrap();

        let hashes: Vec<&str> = infos.iter().map(|i| i.sha256.as_str()).collect();
        let found = db.get_informations_by_sha256s(&hashes).unwrap();
        assert_eq!(found.len(), 5);
    }

    #[test]
    fn update_replaces_existing() {
        let db = SongInformationAccessor::open_in_memory().unwrap();
        let mut info = sample_info();

        db.update(&info).unwrap();

        info.n = 999;
        info.density = 20.0;
        info.peakdensity = 25.0;
        db.update(&info).unwrap();

        let found = db.get_information(&"a".repeat(64)).unwrap().unwrap();
        assert_eq!(found.n, 999);
        assert!((found.density - 20.0).abs() < f64::EPSILON);
        assert!((found.peakdensity - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn chunked_query_large_batch() {
        let db = SongInformationAccessor::open_in_memory().unwrap();

        let mut infos = Vec::new();
        for i in 0..50 {
            let mut info = sample_info();
            info.sha256 = format!("{:0>64}", i);
            infos.push(info);
        }

        db.batch_update(&infos).unwrap();

        let hashes: Vec<&str> = infos.iter().map(|i| i.sha256.as_str()).collect();
        let found = db.get_informations_by_sha256s(&hashes).unwrap();
        assert_eq!(found.len(), 50);
    }

    #[test]
    fn invalid_info_filtered_on_get() {
        let db = SongInformationAccessor::open_in_memory().unwrap();
        // Insert an info with invalid sha256 (too short)
        let info = SongInformation {
            sha256: "short".to_string(),
            n: 10,
            ..Default::default()
        };
        db.update(&info).unwrap();

        let found = db.get_information("short").unwrap();
        assert!(found.is_none()); // Should be filtered by validate()
    }
}

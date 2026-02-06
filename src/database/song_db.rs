use anyhow::Result;
use rusqlite::{Connection, params};

use super::models::{FolderData, SongData};

/// Song database accessor using SQLite.
/// Compatible with beatoraja's songdata.db schema.
pub struct SongDatabase {
    conn: Connection,
}

impl SongDatabase {
    /// Open or create a song database at the given path.
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
            "CREATE TABLE IF NOT EXISTS song (
                md5 TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                subtitle TEXT NOT NULL DEFAULT '',
                genre TEXT NOT NULL DEFAULT '',
                artist TEXT NOT NULL DEFAULT '',
                subartist TEXT NOT NULL DEFAULT '',
                tag TEXT NOT NULL DEFAULT '',
                path TEXT NOT NULL DEFAULT '',
                folder TEXT NOT NULL DEFAULT '',
                stagefile TEXT NOT NULL DEFAULT '',
                banner TEXT NOT NULL DEFAULT '',
                backbmp TEXT NOT NULL DEFAULT '',
                preview TEXT NOT NULL DEFAULT '',
                parent TEXT NOT NULL DEFAULT '',
                level INTEGER NOT NULL DEFAULT 0,
                difficulty INTEGER NOT NULL DEFAULT 0,
                maxbpm INTEGER NOT NULL DEFAULT 0,
                minbpm INTEGER NOT NULL DEFAULT 0,
                length INTEGER NOT NULL DEFAULT 0,
                mode INTEGER NOT NULL DEFAULT 0,
                judge INTEGER NOT NULL DEFAULT 0,
                feature INTEGER NOT NULL DEFAULT 0,
                content INTEGER NOT NULL DEFAULT 0,
                date INTEGER NOT NULL DEFAULT 0,
                favorite INTEGER NOT NULL DEFAULT 0,
                adddate INTEGER NOT NULL DEFAULT 0,
                notes INTEGER NOT NULL DEFAULT 0,
                charthash TEXT NOT NULL DEFAULT '',
                UNIQUE(path)
            );
            CREATE INDEX IF NOT EXISTS idx_song_md5 ON song(md5);
            CREATE INDEX IF NOT EXISTS idx_song_sha256 ON song(sha256);
            CREATE INDEX IF NOT EXISTS idx_song_parent ON song(parent);
            CREATE INDEX IF NOT EXISTS idx_song_folder ON song(folder);

            CREATE TABLE IF NOT EXISTS folder (
                title TEXT NOT NULL DEFAULT '',
                subtitle TEXT NOT NULL DEFAULT '',
                command TEXT NOT NULL DEFAULT '',
                path TEXT NOT NULL DEFAULT '',
                banner TEXT NOT NULL DEFAULT '',
                parent TEXT NOT NULL DEFAULT '',
                type INTEGER NOT NULL DEFAULT 0,
                date INTEGER NOT NULL DEFAULT 0,
                adddate INTEGER NOT NULL DEFAULT 0,
                max INTEGER NOT NULL DEFAULT 0,
                UNIQUE(path)
            );
            CREATE INDEX IF NOT EXISTS idx_folder_parent ON folder(parent);",
        )?;
        Ok(())
    }

    /// Insert or replace a song record.
    pub fn upsert_song(&self, song: &SongData) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO song
                (md5, sha256, title, subtitle, genre, artist, subartist, tag, path, folder,
                 stagefile, banner, backbmp, preview, parent, level, difficulty, maxbpm, minbpm,
                 length, mode, judge, feature, content, date, favorite, adddate, notes, charthash)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29)",
            params![
                song.md5,
                song.sha256,
                song.title,
                song.subtitle,
                song.genre,
                song.artist,
                song.subartist,
                song.tag,
                song.path,
                song.folder,
                song.stagefile,
                song.banner,
                song.backbmp,
                song.preview,
                song.parent,
                song.level,
                song.difficulty,
                song.maxbpm,
                song.minbpm,
                song.length,
                song.mode,
                song.judge,
                song.feature,
                song.content,
                song.date,
                song.favorite,
                song.adddate,
                song.notes,
                song.charthash,
            ],
        )?;
        Ok(())
    }

    /// Insert or replace a folder record.
    pub fn upsert_folder(&self, folder: &FolderData) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO folder
                (title, subtitle, command, path, banner, parent, type, date, adddate, max)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                folder.title,
                folder.subtitle,
                folder.command,
                folder.path,
                folder.banner,
                folder.parent,
                folder.folder_type,
                folder.date,
                folder.adddate,
                folder.max,
            ],
        )?;
        Ok(())
    }

    /// Get songs by a column value.
    pub fn get_songs_by(&self, key: &str, value: &str) -> Result<Vec<SongData>> {
        // Only allow safe column names.
        let allowed = [
            "md5", "sha256", "path", "parent", "folder", "title", "artist", "genre",
        ];
        if !allowed.contains(&key) {
            anyhow::bail!("Invalid column name: {}", key);
        }
        let sql = format!("SELECT * FROM song WHERE {} = ?1", key);
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![value], SongData::from_row)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Get songs by SHA256 hash.
    pub fn get_song_by_sha256(&self, sha256: &str) -> Result<Vec<SongData>> {
        self.get_songs_by("sha256", sha256)
    }

    /// Get songs by MD5 hash.
    pub fn get_song_by_md5(&self, md5: &str) -> Result<Vec<SongData>> {
        self.get_songs_by("md5", md5)
    }

    /// Get songs by multiple hashes (MD5 or SHA256 determined by length).
    pub fn get_songs_by_hashes(&self, hashes: &[&str]) -> Result<Vec<SongData>> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }
        let mut md5_hashes = Vec::new();
        let mut sha256_hashes = Vec::new();
        for h in hashes {
            if h.len() > 32 {
                sha256_hashes.push(*h);
            } else {
                md5_hashes.push(*h);
            }
        }

        let mut results = Vec::new();

        if !sha256_hashes.is_empty() {
            let placeholders: Vec<String> = (0..sha256_hashes.len())
                .map(|i| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "SELECT * FROM song WHERE sha256 IN ({})",
                placeholders.join(",")
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::types::ToSql> = sha256_hashes
                .iter()
                .map(|h| h as &dyn rusqlite::types::ToSql)
                .collect();
            let rows = stmt.query_map(params.as_slice(), SongData::from_row)?;
            for row in rows {
                results.push(row?);
            }
        }

        if !md5_hashes.is_empty() {
            let placeholders: Vec<String> = (0..md5_hashes.len())
                .map(|i| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "SELECT * FROM song WHERE md5 IN ({})",
                placeholders.join(",")
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let params: Vec<&dyn rusqlite::types::ToSql> = md5_hashes
                .iter()
                .map(|h| h as &dyn rusqlite::types::ToSql)
                .collect();
            let rows = stmt.query_map(params.as_slice(), SongData::from_row)?;
            for row in rows {
                results.push(row?);
            }
        }

        Ok(results)
    }

    /// Full-text search across title, subtitle, artist, subartist, and genre.
    pub fn search_songs(&self, text: &str) -> Result<Vec<SongData>> {
        let pattern = format!("%{}%", text);
        let mut stmt = self.conn.prepare(
            "SELECT * FROM song
             WHERE rtrim(title||' '||subtitle||' '||artist||' '||subartist||' '||genre) LIKE ?1
             GROUP BY sha256",
        )?;
        let rows = stmt.query_map(params![pattern], SongData::from_row)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Get folders by a column value.
    pub fn get_folders_by(&self, key: &str, value: &str) -> Result<Vec<FolderData>> {
        let allowed = ["path", "parent", "title"];
        if !allowed.contains(&key) {
            anyhow::bail!("Invalid column name: {}", key);
        }
        let sql = format!("SELECT * FROM folder WHERE {} = ?1", key);
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![value], FolderData::from_row)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Delete songs whose path starts with the given prefix.
    pub fn delete_songs_by_path_prefix(&self, prefix: &str) -> Result<usize> {
        let pattern = format!("{}%", prefix);
        let count = self
            .conn
            .execute("DELETE FROM song WHERE path LIKE ?1", params![pattern])?;
        Ok(count)
    }

    /// Delete folders whose path starts with the given prefix.
    pub fn delete_folders_by_path_prefix(&self, prefix: &str) -> Result<usize> {
        let pattern = format!("{}%", prefix);
        let count = self
            .conn
            .execute("DELETE FROM folder WHERE path LIKE ?1", params![pattern])?;
        Ok(count)
    }

    /// Delete a song by exact path.
    pub fn delete_song_by_path(&self, path: &str) -> Result<usize> {
        let count = self
            .conn
            .execute("DELETE FROM song WHERE path = ?1", params![path])?;
        Ok(count)
    }

    /// Update the favorite flag for a song by SHA256.
    pub fn set_favorite(&self, sha256: &str, favorite: i32) -> Result<()> {
        self.conn.execute(
            "UPDATE song SET favorite = ?1 WHERE sha256 = ?2",
            params![favorite, sha256],
        )?;
        Ok(())
    }

    /// Get all songs count.
    pub fn song_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM song", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Begin a transaction for batch operations.
    /// Returns a guard that commits on drop.
    pub fn begin_batch(&mut self) -> Result<BatchGuard<'_>> {
        self.conn.execute_batch("BEGIN TRANSACTION")?;
        Ok(BatchGuard { conn: &self.conn })
    }
}

/// RAII guard for batch database operations.
/// Commits the transaction when dropped.
pub struct BatchGuard<'a> {
    conn: &'a Connection,
}

impl<'a> BatchGuard<'a> {
    /// Commit the transaction explicitly.
    pub fn commit(self) -> Result<()> {
        self.conn.execute_batch("COMMIT")?;
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for BatchGuard<'_> {
    fn drop(&mut self) {
        let _ = self.conn.execute_batch("ROLLBACK");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::SongData;

    fn make_song(title: &str, sha256: &str, md5: &str, path: &str) -> SongData {
        SongData {
            title: title.to_string(),
            sha256: sha256.to_string(),
            md5: md5.to_string(),
            path: path.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn create_and_query_song() {
        let db = SongDatabase::open_in_memory().unwrap();
        let song = make_song("Test Song", "abc123sha256", "abc123md5", "test/song.bms");
        db.upsert_song(&song).unwrap();

        let results = db.get_song_by_sha256("abc123sha256").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Song");
        assert_eq!(results[0].path, "test/song.bms");
    }

    #[test]
    fn upsert_replaces_by_path() {
        let db = SongDatabase::open_in_memory().unwrap();
        let song1 = make_song("Song V1", "sha1", "md1", "same/path.bms");
        db.upsert_song(&song1).unwrap();

        let song2 = make_song("Song V2", "sha2", "md2", "same/path.bms");
        db.upsert_song(&song2).unwrap();

        assert_eq!(db.song_count().unwrap(), 1);
        let results = db.get_song_by_sha256("sha2").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Song V2");
    }

    #[test]
    fn search_songs_by_text() {
        let db = SongDatabase::open_in_memory().unwrap();
        db.upsert_song(&SongData {
            title: "Colorful".to_string(),
            artist: "dj TAKA".to_string(),
            sha256: "s1".to_string(),
            path: "p1".to_string(),
            ..Default::default()
        })
        .unwrap();
        db.upsert_song(&SongData {
            title: "Another".to_string(),
            artist: "kors k".to_string(),
            sha256: "s2".to_string(),
            path: "p2".to_string(),
            ..Default::default()
        })
        .unwrap();

        let results = db.search_songs("TAKA").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Colorful");
    }

    #[test]
    fn query_by_hashes() {
        let db = SongDatabase::open_in_memory().unwrap();
        db.upsert_song(&make_song(
            "Song A",
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
            "md5a",
            "pa",
        ))
        .unwrap();
        db.upsert_song(&make_song(
            "Song B",
            "short",
            "md5bmd5bmd5bmd5bmd5bmd5bmd5bmd5b",
            "pb",
        ))
        .unwrap();

        let results = db
            .get_songs_by_hashes(&[
                "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
                "md5bmd5bmd5bmd5bmd5bmd5bmd5bmd5b",
            ])
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn folder_crud() {
        let db = SongDatabase::open_in_memory().unwrap();
        let folder = FolderData {
            title: "BMS Collection".to_string(),
            path: "bms/collection/".to_string(),
            parent: "root_crc".to_string(),
            ..Default::default()
        };
        db.upsert_folder(&folder).unwrap();

        let results = db.get_folders_by("parent", "root_crc").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "BMS Collection");
    }

    #[test]
    fn delete_by_path_prefix() {
        let db = SongDatabase::open_in_memory().unwrap();
        db.upsert_song(&make_song("S1", "sha1", "md1", "bms/a/1.bms"))
            .unwrap();
        db.upsert_song(&make_song("S2", "sha2", "md2", "bms/a/2.bms"))
            .unwrap();
        db.upsert_song(&make_song("S3", "sha3", "md3", "bms/b/1.bms"))
            .unwrap();

        let deleted = db.delete_songs_by_path_prefix("bms/a/").unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(db.song_count().unwrap(), 1);
    }

    #[test]
    fn set_favorite() {
        let db = SongDatabase::open_in_memory().unwrap();
        db.upsert_song(&make_song("Song", "sha_fav", "md_fav", "fav.bms"))
            .unwrap();

        db.set_favorite("sha_fav", 1).unwrap();
        let results = db.get_song_by_sha256("sha_fav").unwrap();
        assert_eq!(results[0].favorite, 1);
    }

    #[test]
    fn invalid_column_rejected() {
        let db = SongDatabase::open_in_memory().unwrap();
        let result = db.get_songs_by("DROP TABLE song; --", "x");
        assert!(result.is_err());
    }
}

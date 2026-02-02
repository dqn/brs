use crate::database::connection::Database;
use crate::database::models::{Mode, SongData};
use anyhow::{Context, Result};
use rusqlite::params;
use std::path::{Path, PathBuf};

/// Accessor for song database operations.
pub struct SongDatabaseAccessor<'a> {
    db: &'a Database,
}

impl<'a> SongDatabaseAccessor<'a> {
    /// Create a new accessor for the given database.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Insert or update a song in the database.
    pub fn insert_song(&self, song: &SongData) -> Result<()> {
        let conn = self.db.connection();

        conn.execute(
            r#"
            INSERT INTO song (
                sha256, md5, path, folder, title, subtitle, artist, subartist,
                genre, mode, level, difficulty, max_bpm, min_bpm, notes, date, add_date
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17
            )
            ON CONFLICT(sha256) DO UPDATE SET
                md5 = excluded.md5,
                path = excluded.path,
                folder = excluded.folder,
                title = excluded.title,
                subtitle = excluded.subtitle,
                artist = excluded.artist,
                subartist = excluded.subartist,
                genre = excluded.genre,
                mode = excluded.mode,
                level = excluded.level,
                difficulty = excluded.difficulty,
                max_bpm = excluded.max_bpm,
                min_bpm = excluded.min_bpm,
                notes = excluded.notes,
                date = excluded.date
            "#,
            params![
                song.sha256,
                song.md5,
                song.path.to_string_lossy(),
                song.folder,
                song.title,
                song.subtitle,
                song.artist,
                song.subartist,
                song.genre,
                song.mode.as_i32(),
                song.level,
                song.difficulty,
                song.max_bpm,
                song.min_bpm,
                song.notes,
                song.date,
                song.add_date,
            ],
        )
        .context("Failed to insert song")?;

        Ok(())
    }

    /// Get a song by its SHA256 hash.
    pub fn get_song_by_sha256(&self, sha256: &str) -> Result<Option<SongData>> {
        let conn = self.db.connection();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT sha256, md5, path, folder, title, subtitle, artist, subartist,
                       genre, mode, level, difficulty, max_bpm, min_bpm, notes, date, add_date
                FROM song WHERE sha256 = ?1
                "#,
            )
            .context("Failed to prepare statement")?;

        let result = stmt.query_row([sha256], |row| {
            Ok(SongData {
                sha256: row.get(0)?,
                md5: row.get(1)?,
                path: PathBuf::from(row.get::<_, String>(2)?),
                folder: row.get(3)?,
                title: row.get(4)?,
                subtitle: row.get(5)?,
                artist: row.get(6)?,
                subartist: row.get(7)?,
                genre: row.get(8)?,
                mode: Mode::from_i32(row.get(9)?).unwrap_or_default(),
                level: row.get(10)?,
                difficulty: row.get(11)?,
                max_bpm: row.get(12)?,
                min_bpm: row.get(13)?,
                notes: row.get(14)?,
                date: row.get(15)?,
                add_date: row.get(16)?,
            })
        });

        match result {
            Ok(song) => Ok(Some(song)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query song by sha256"),
        }
    }

    /// Get a song by its file path.
    pub fn get_song_by_path(&self, path: &Path) -> Result<Option<SongData>> {
        let conn = self.db.connection();
        let path_str = path.to_string_lossy();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT sha256, md5, path, folder, title, subtitle, artist, subartist,
                       genre, mode, level, difficulty, max_bpm, min_bpm, notes, date, add_date
                FROM song WHERE path = ?1
                "#,
            )
            .context("Failed to prepare statement")?;

        let result = stmt.query_row([path_str.as_ref()], |row| {
            Ok(SongData {
                sha256: row.get(0)?,
                md5: row.get(1)?,
                path: PathBuf::from(row.get::<_, String>(2)?),
                folder: row.get(3)?,
                title: row.get(4)?,
                subtitle: row.get(5)?,
                artist: row.get(6)?,
                subartist: row.get(7)?,
                genre: row.get(8)?,
                mode: Mode::from_i32(row.get(9)?).unwrap_or_default(),
                level: row.get(10)?,
                difficulty: row.get(11)?,
                max_bpm: row.get(12)?,
                min_bpm: row.get(13)?,
                notes: row.get(14)?,
                date: row.get(15)?,
                add_date: row.get(16)?,
            })
        });

        match result {
            Ok(song) => Ok(Some(song)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query song by path"),
        }
    }

    /// Get all songs from the database.
    pub fn get_all_songs(&self) -> Result<Vec<SongData>> {
        let conn = self.db.connection();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT sha256, md5, path, folder, title, subtitle, artist, subartist,
                       genre, mode, level, difficulty, max_bpm, min_bpm, notes, date, add_date
                FROM song ORDER BY add_date DESC
                "#,
            )
            .context("Failed to prepare statement")?;

        let songs = stmt
            .query_map([], |row| {
                Ok(SongData {
                    sha256: row.get(0)?,
                    md5: row.get(1)?,
                    path: PathBuf::from(row.get::<_, String>(2)?),
                    folder: row.get(3)?,
                    title: row.get(4)?,
                    subtitle: row.get(5)?,
                    artist: row.get(6)?,
                    subartist: row.get(7)?,
                    genre: row.get(8)?,
                    mode: Mode::from_i32(row.get(9)?).unwrap_or_default(),
                    level: row.get(10)?,
                    difficulty: row.get(11)?,
                    max_bpm: row.get(12)?,
                    min_bpm: row.get(13)?,
                    notes: row.get(14)?,
                    date: row.get(15)?,
                    add_date: row.get(16)?,
                })
            })
            .context("Failed to query all songs")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect songs")?;

        Ok(songs)
    }

    /// Delete a song by its SHA256 hash.
    pub fn delete_song(&self, sha256: &str) -> Result<bool> {
        let conn = self.db.connection();

        let rows_affected = conn
            .execute("DELETE FROM song WHERE sha256 = ?1", [sha256])
            .context("Failed to delete song")?;

        Ok(rows_affected > 0)
    }

    /// Delete all songs in a folder.
    pub fn delete_songs_by_folder(&self, folder: &str) -> Result<usize> {
        let conn = self.db.connection();

        let rows_affected = conn
            .execute("DELETE FROM song WHERE folder = ?1", [folder])
            .context("Failed to delete songs by folder")?;

        Ok(rows_affected)
    }

    /// Get the number of songs in the database.
    pub fn count(&self) -> Result<usize> {
        let conn = self.db.connection();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM song", [], |row| row.get(0))
            .context("Failed to count songs")?;

        Ok(count as usize)
    }

    /// Check if a song exists by its SHA256 hash.
    pub fn exists(&self, sha256: &str) -> Result<bool> {
        let conn = self.db.connection();

        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM song WHERE sha256 = ?1",
                [sha256],
                |row| row.get(0),
            )
            .context("Failed to check song existence")?;

        Ok(exists > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::Database;

    fn create_test_song(sha256: &str) -> SongData {
        let mut song = SongData::new(sha256.to_string(), PathBuf::from("/test/song.bms"));
        song.title = "Test Song".to_string();
        song.artist = "Test Artist".to_string();
        song.genre = "Test Genre".to_string();
        song.level = 7;
        song.notes = 1000;
        song.folder = "test".to_string();
        song
    }

    #[test]
    fn test_insert_and_get_song() {
        let db = Database::open_song_db_in_memory().unwrap();
        let accessor = SongDatabaseAccessor::new(&db);

        let song = create_test_song("abc123");
        accessor.insert_song(&song).unwrap();

        let retrieved = accessor.get_song_by_sha256("abc123").unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.title, "Test Song");
        assert_eq!(retrieved.artist, "Test Artist");
    }

    #[test]
    fn test_get_song_by_path() {
        let db = Database::open_song_db_in_memory().unwrap();
        let accessor = SongDatabaseAccessor::new(&db);

        let song = create_test_song("abc123");
        accessor.insert_song(&song).unwrap();

        let retrieved = accessor
            .get_song_by_path(Path::new("/test/song.bms"))
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().sha256, "abc123");
    }

    #[test]
    fn test_get_all_songs() {
        let db = Database::open_song_db_in_memory().unwrap();
        let accessor = SongDatabaseAccessor::new(&db);

        accessor.insert_song(&create_test_song("abc")).unwrap();
        accessor.insert_song(&create_test_song("def")).unwrap();
        accessor.insert_song(&create_test_song("ghi")).unwrap();

        let songs = accessor.get_all_songs().unwrap();
        assert_eq!(songs.len(), 3);
    }

    #[test]
    fn test_delete_song() {
        let db = Database::open_song_db_in_memory().unwrap();
        let accessor = SongDatabaseAccessor::new(&db);

        let song = create_test_song("abc123");
        accessor.insert_song(&song).unwrap();

        assert!(accessor.delete_song("abc123").unwrap());
        assert!(accessor.get_song_by_sha256("abc123").unwrap().is_none());
    }

    #[test]
    fn test_delete_songs_by_folder() {
        let db = Database::open_song_db_in_memory().unwrap();
        let accessor = SongDatabaseAccessor::new(&db);

        let mut song1 = create_test_song("abc");
        song1.folder = "folder1".to_string();
        let mut song2 = create_test_song("def");
        song2.folder = "folder1".to_string();
        let mut song3 = create_test_song("ghi");
        song3.folder = "folder2".to_string();

        accessor.insert_song(&song1).unwrap();
        accessor.insert_song(&song2).unwrap();
        accessor.insert_song(&song3).unwrap();

        let deleted = accessor.delete_songs_by_folder("folder1").unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(accessor.count().unwrap(), 1);
    }

    #[test]
    fn test_upsert_song() {
        let db = Database::open_song_db_in_memory().unwrap();
        let accessor = SongDatabaseAccessor::new(&db);

        let mut song = create_test_song("abc123");
        accessor.insert_song(&song).unwrap();

        song.title = "Updated Title".to_string();
        accessor.insert_song(&song).unwrap();

        let retrieved = accessor.get_song_by_sha256("abc123").unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert_eq!(accessor.count().unwrap(), 1);
    }

    #[test]
    fn test_exists() {
        let db = Database::open_song_db_in_memory().unwrap();
        let accessor = SongDatabaseAccessor::new(&db);

        assert!(!accessor.exists("abc123").unwrap());

        accessor.insert_song(&create_test_song("abc123")).unwrap();
        assert!(accessor.exists("abc123").unwrap());
    }
}

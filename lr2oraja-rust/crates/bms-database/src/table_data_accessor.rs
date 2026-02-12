//! Local caching accessor for difficulty table data.
//!
//! Port of Java `TableDataAccessor.java`.
//! Uses SHA-256 hash of URL as filename for cached .bmt files.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::table_data::TableData;

/// Accessor for cached difficulty table data stored as .bmt files.
pub struct TableDataAccessor {
    table_dir: PathBuf,
}

impl TableDataAccessor {
    /// Create a new accessor for the given directory, creating it if needed.
    pub fn new(table_dir: impl Into<PathBuf>) -> Result<Self> {
        let table_dir = table_dir.into();
        if !table_dir.exists() {
            fs::create_dir_all(&table_dir)?;
        }
        Ok(Self { table_dir })
    }

    /// Read all cached table data (.bmt files) from the directory.
    pub fn read_all(&self) -> Result<Vec<TableData>> {
        let mut results = Vec::new();
        let entries = fs::read_dir(&self.table_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "bmt") {
                match TableData::read(&path) {
                    Ok(td) => results.push(td),
                    Err(_) => continue,
                }
            }
        }
        Ok(results)
    }

    /// Read a cached table by URL.
    ///
    /// Returns `Ok(None)` if no cache exists for the URL.
    pub fn read_cache(&self, url: &str) -> Result<Option<TableData>> {
        let filename = format!("{}.bmt", Self::get_filename(url));
        let path = self.table_dir.join(filename);
        if !path.exists() {
            return Ok(None);
        }
        match TableData::read(&path) {
            Ok(td) => Ok(Some(td)),
            Err(_) => Ok(None),
        }
    }

    /// Write table data to cache using SHA-256(url) as filename.
    pub fn write(&self, data: &TableData) -> Result<()> {
        let filename = format!("{}.bmt", Self::get_filename(&data.url));
        let path = self.table_dir.join(filename);
        TableData::write(&path, data)
    }

    /// Get the SHA-256 hex hash of a URL string (used as cache filename).
    pub fn get_filename(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let result = hasher.finalize();
        hex_encode(&result)
    }
}

/// Encode bytes as lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course_data::CourseSongData;
    use crate::table_data::TableFolder;

    fn sample_table(url: &str, name: &str) -> TableData {
        TableData {
            url: url.to_string(),
            name: name.to_string(),
            tag: "T".to_string(),
            folder: vec![TableFolder {
                name: "Level 1".to_string(),
                songs: vec![CourseSongData {
                    sha256: "abc".to_string(),
                    md5: String::new(),
                    title: "Song".to_string(),
                }],
            }],
            course: Vec::new(),
        }
    }

    #[test]
    fn get_filename_deterministic() {
        let hash1 = TableDataAccessor::get_filename("https://example.com/table");
        let hash2 = TableDataAccessor::get_filename("https://example.com/table");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn get_filename_different_for_different_urls() {
        let hash1 = TableDataAccessor::get_filename("https://example.com/a");
        let hash2 = TableDataAccessor::get_filename("https://example.com/b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn write_and_read_cache() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = TableDataAccessor::new(dir.path()).unwrap();

        let td = sample_table("https://example.com/table", "Test Table");
        accessor.write(&td).unwrap();

        let cached = accessor
            .read_cache("https://example.com/table")
            .unwrap()
            .unwrap();
        assert_eq!(cached.name, "Test Table");
        assert_eq!(cached.url, "https://example.com/table");
    }

    #[test]
    fn read_cache_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = TableDataAccessor::new(dir.path()).unwrap();
        let result = accessor.read_cache("https://nonexistent.com").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn read_all_tables() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = TableDataAccessor::new(dir.path()).unwrap();

        accessor
            .write(&sample_table("https://a.com", "Table A"))
            .unwrap();
        accessor
            .write(&sample_table("https://b.com", "Table B"))
            .unwrap();

        let tables = accessor.read_all().unwrap();
        assert_eq!(tables.len(), 2);
    }

    #[test]
    fn creates_directory_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("tables").join("cache");
        let accessor = TableDataAccessor::new(&nested).unwrap();
        assert!(nested.exists());
        let tables = accessor.read_all().unwrap();
        assert!(tables.is_empty());
    }

    #[test]
    fn skips_non_bmt_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("readme.txt"), "not a table").unwrap();
        fs::write(dir.path().join("data.json"), "{}").unwrap();

        let accessor = TableDataAccessor::new(dir.path()).unwrap();
        accessor
            .write(&sample_table("https://a.com", "Table A"))
            .unwrap();

        let tables = accessor.read_all().unwrap();
        assert_eq!(tables.len(), 1);
    }

    #[test]
    fn overwrite_existing_cache() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = TableDataAccessor::new(dir.path()).unwrap();
        let url = "https://example.com/table";

        accessor.write(&sample_table(url, "Version 1")).unwrap();
        accessor.write(&sample_table(url, "Version 2")).unwrap();

        let cached = accessor.read_cache(url).unwrap().unwrap();
        assert_eq!(cached.name, "Version 2");
    }
}

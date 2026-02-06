use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

use super::models::{FolderData, SongData};
use super::song_db::SongDatabase;
use crate::model::bms_loader;
use crate::state::select::bar_manager::crc32_folder;

/// BMS file extensions to scan.
const BMS_EXTENSIONS: &[&str] = &["bms", "bme", "bml", "bmson"];

/// Scan BMS directories and populate the song database.
/// Skips files already registered in the database and cleans up stale entries.
pub fn scan_directories(song_db: &SongDatabase, roots: &[String]) -> Result<usize> {
    let known_paths = song_db.get_all_paths()?;
    let mut scanned_paths = HashSet::new();
    let mut count = 0;

    let batch = song_db.begin_batch()?;

    for root in roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            tracing::warn!("BMS directory not found: {root}");
            continue;
        }
        count += scan_directory(song_db, root_path, root, &known_paths, &mut scanned_paths)?;
    }

    // Clean up stale entries (files that no longer exist on disk).
    let stale_count = song_db.delete_songs_not_in(&scanned_paths)?;
    if stale_count > 0 {
        tracing::info!("removed {stale_count} stale song entries");
    }

    batch.commit()?;

    Ok(count)
}

/// Recursively scan a single directory.
fn scan_directory(
    song_db: &SongDatabase,
    dir: &Path,
    root: &str,
    known_paths: &HashSet<String>,
    scanned_paths: &mut HashSet<String>,
) -> Result<usize> {
    let mut count = 0;
    let parent_crc = crc32_folder(dir.to_string_lossy().as_ref());

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("cannot read directory {}: {e}", dir.display());
            return Ok(0);
        }
    };

    let mut has_bms_files = false;
    let mut subdirs = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            subdirs.push(path);
        } else if is_bms_file(&path) {
            has_bms_files = true;
            let path_str = path.to_string_lossy().to_string();
            scanned_paths.insert(path_str.clone());

            // Skip files already registered in the database.
            if known_paths.contains(&path_str) {
                continue;
            }

            if let Err(e) = register_bms(song_db, &path, &parent_crc) {
                tracing::debug!("skip {}: {e}", path.display());
            } else {
                count += 1;
            }
        }
    }

    // Register this directory as a folder if it has BMS files or subdirs with BMS files.
    if has_bms_files {
        let title = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let dir_str = dir.to_string_lossy().to_string();
        let root_crc = crc32_folder(root);

        song_db.upsert_folder(&FolderData {
            title,
            path: dir_str,
            parent: root_crc,
            ..Default::default()
        })?;
    }

    for subdir in subdirs {
        count += scan_directory(song_db, &subdir, root, known_paths, scanned_paths)?;
    }

    Ok(count)
}

/// Check if a path has a BMS file extension.
fn is_bms_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| BMS_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

/// Parse a BMS file and register it in the database.
/// Reads the file once and reuses the bytes for both parsing and hashing.
fn register_bms(song_db: &SongDatabase, path: &Path, parent_crc: &str) -> Result<()> {
    let content = std::fs::read(path)?;

    let model = bms_loader::load_bms_from_bytes(&content, path)?;

    let sha256 = format!("{:x}", Sha256::digest(&content));
    let md5 = format!("{:x}", md5::compute(&content));

    let path_str = path.to_string_lossy().to_string();
    let folder = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let total_notes = model.notes.len() as i32;
    let max_bpm = model.bpm as i32;

    song_db.upsert_song(&SongData {
        md5,
        sha256,
        title: model.title,
        subtitle: model.subtitle,
        genre: model.genre,
        artist: model.artist,
        subartist: model.subartist,
        path: path_str,
        folder,
        stagefile: model.stagefile,
        banner: model.banner,
        backbmp: model.back_bmp,
        preview: model.preview,
        parent: parent_crc.to_string(),
        level: model.playlevel.parse().unwrap_or(0),
        difficulty: model.difficulty,
        maxbpm: max_bpm,
        minbpm: max_bpm,
        mode: model.mode as i32,
        judge: model.judge_rank,
        notes: total_notes,
        ..Default::default()
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_bms_file_extensions() {
        assert!(is_bms_file(Path::new("test.bms")));
        assert!(is_bms_file(Path::new("test.BMS")));
        assert!(is_bms_file(Path::new("test.bme")));
        assert!(is_bms_file(Path::new("test.bml")));
        assert!(is_bms_file(Path::new("test.bmson")));
        assert!(!is_bms_file(Path::new("test.txt")));
        assert!(!is_bms_file(Path::new("test.mp3")));
    }

    #[test]
    fn scan_nonexistent_directory() {
        let db = SongDatabase::open_in_memory().unwrap();
        let count = scan_directories(&db, &["nonexistent_dir_12345".to_string()]).unwrap();
        assert_eq!(count, 0);
    }
}

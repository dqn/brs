use crate::database::models::{Mode, ScanResult, SongData};
use crate::database::song_db::SongDatabaseAccessor;
use anyhow::{Context, Result};
use bms_rs::bms::prelude::*;
use encoding_rs::{SHIFT_JIS, UTF_8};
use md5::{Digest as Md5Digest, Md5};
use num_traits::ToPrimitive;
use sha2::Sha256;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

/// BMS file extensions to scan.
const BMS_EXTENSIONS: &[&str] = &["bms", "bme", "bml", "pms"];

/// Song scanner for BMS folders.
pub struct SongScanner {
    root_path: PathBuf,
}

impl SongScanner {
    /// Create a new scanner for the given root path.
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    /// Scan the folder and update the song database.
    pub fn scan_folder<F>(
        &self,
        accessor: &SongDatabaseAccessor,
        mut progress: F,
    ) -> Result<ScanResult>
    where
        F: FnMut(&str),
    {
        let mut result = ScanResult::new();
        let mut found_hashes = HashSet::new();

        // Walk the directory tree
        for entry in WalkDir::new(&self.root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip non-BMS files
            if !is_bms_file(path) {
                continue;
            }

            progress(&format!("Scanning: {}", path.display()));

            match self.process_bms_file(path, accessor) {
                Ok((song_data, is_new)) => {
                    found_hashes.insert(song_data.sha256.clone());
                    if is_new {
                        result.added += 1;
                    } else {
                        result.updated += 1;
                    }
                }
                Err(e) => {
                    result.errors.push((path.to_path_buf(), e.to_string()));
                }
            }
        }

        // Check for deleted songs (songs in DB but not found in scan)
        let all_songs = accessor.get_all_songs().unwrap_or_default();
        for song in all_songs {
            if song.path.starts_with(&self.root_path)
                && !found_hashes.contains(&song.sha256)
                && accessor.delete_song(&song.sha256).unwrap_or(false)
            {
                result.deleted += 1;
            }
        }

        Ok(result)
    }

    /// Process a single BMS file and update the database.
    fn process_bms_file(
        &self,
        path: &Path,
        accessor: &SongDatabaseAccessor,
    ) -> Result<(SongData, bool)> {
        // Calculate hashes
        let bytes = fs::read(path).context("Failed to read file")?;
        let sha256 = calc_sha256(&bytes);
        let md5 = calc_md5(&bytes);

        // Check if already exists
        let existing = accessor.get_song_by_sha256(&sha256)?;
        let is_new = existing.is_none();

        // Parse BMS header
        let song_data = self.parse_bms_header(path, &bytes, sha256, md5)?;

        // Insert or update
        accessor.insert_song(&song_data)?;

        Ok((song_data, is_new))
    }

    /// Parse BMS file header to extract metadata.
    fn parse_bms_header(
        &self,
        path: &Path,
        bytes: &[u8],
        sha256: String,
        md5: String,
    ) -> Result<SongData> {
        let source = decode_bms_content(bytes);
        let BmsOutput { bms, warnings: _ } = parse_bms(&source, default_config())
            .map_err(|e| anyhow::anyhow!("BMS parse error: {:?}", e))?;

        let folder = path
            .parent()
            .and_then(|p| p.strip_prefix(&self.root_path).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_date = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Count notes
        let notes = count_notes(&bms);

        // Get BPM
        let initial_bpm = bms
            .bpm
            .bpm
            .as_ref()
            .and_then(|d| d.to_f64())
            .unwrap_or(120.0);

        // Calculate min/max BPM
        let (min_bpm, max_bpm) = calc_bpm_range(&bms, initial_bpm);

        let mut song = SongData::new(sha256, path.to_path_buf());
        song.md5 = md5;
        song.folder = folder;
        song.title = bms.music_info.title.clone().unwrap_or_default();
        song.subtitle = bms.music_info.subtitle.clone().unwrap_or_default();
        song.artist = bms.music_info.artist.clone().unwrap_or_default();
        song.subartist = bms.music_info.sub_artist.clone().unwrap_or_default();
        song.genre = bms.music_info.genre.clone().unwrap_or_default();
        song.mode = detect_mode(&bms);
        song.level = bms.metadata.play_level.map(|d| d as i32).unwrap_or(0);
        song.difficulty = bms.metadata.difficulty.map(|d| d as i32).unwrap_or(0);
        song.max_bpm = max_bpm as i32;
        song.min_bpm = min_bpm as i32;
        song.notes = notes as i32;
        song.date = file_date;
        song.add_date = now;

        Ok(song)
    }
}

/// Calculate SHA256 hash of data.
pub fn calc_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Calculate MD5 hash of data.
pub fn calc_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Check if a path is a BMS file.
fn is_bms_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| BMS_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Decode BMS file content, trying UTF-8 first then Shift-JIS.
fn decode_bms_content(bytes: &[u8]) -> String {
    let (cow, _, had_errors) = UTF_8.decode(bytes);
    if !had_errors {
        return cow.into_owned();
    }

    let (cow, _, _) = SHIFT_JIS.decode(bytes);
    cow.into_owned()
}

/// Count playable notes in a BMS.
fn count_notes(bms: &Bms) -> usize {
    use bms_rs::bms::command::channel::NoteKind;
    use bms_rs::bms::command::channel::mapper::{KeyLayoutBeat, KeyMapping};

    bms.notes()
        .all_notes()
        .filter(|note| {
            // Skip null wav IDs
            if note.wav_id.is_null() {
                return false;
            }

            // Get the note kind
            let Some(layout) = KeyLayoutBeat::from_channel_id(note.channel_id) else {
                return false;
            };

            // Only count visible and long notes
            matches!(layout.kind(), NoteKind::Visible | NoteKind::Long)
        })
        .count()
}

/// Calculate BPM range from BMS.
fn calc_bpm_range(bms: &Bms, initial_bpm: f64) -> (f64, f64) {
    let mut min_bpm = initial_bpm;
    let mut max_bpm = initial_bpm;

    for change in bms.bpm.bpm_changes.values() {
        if let Some(bpm) = change.bpm.to_f64() {
            if bpm > 0.0 {
                min_bpm = min_bpm.min(bpm);
                max_bpm = max_bpm.max(bpm);
            }
        }
    }

    (min_bpm, max_bpm)
}

/// Detect play mode from BMS.
fn detect_mode(bms: &Bms) -> Mode {
    use bms_rs::bms::command::channel::mapper::{KeyLayoutBeat, KeyMapping};
    use bms_rs::bms::prelude::Key;

    let mut has_key6 = false;
    let mut has_key7 = false;
    let mut has_2p = false;

    for note in bms.notes().all_notes() {
        if let Some(layout) = KeyLayoutBeat::from_channel_id(note.channel_id) {
            let key = layout.key();
            match key {
                Key::Key(6) => has_key6 = true,
                Key::Key(7) => has_key7 = true,
                Key::Key(n) if n > 7 => has_2p = true,
                Key::Scratch(player) if player > 1 => has_2p = true,
                _ => {}
            }
        }
    }

    if has_2p {
        if has_key7 {
            Mode::Beat14K
        } else {
            Mode::Beat10K
        }
    } else if has_key7 || has_key6 {
        // 6-key is often played as 7K
        Mode::Beat7K
    } else {
        Mode::Beat5K
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_sha256() {
        let data = b"Hello, World!";
        let hash = calc_sha256(data);
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_calc_md5() {
        let data = b"Hello, World!";
        let hash = calc_md5(data);
        assert_eq!(hash.len(), 32);
        assert_eq!(hash, "65a8e27d8879283831b664bd8b7f0ad4");
    }

    #[test]
    fn test_is_bms_file() {
        assert!(is_bms_file(Path::new("/path/to/song.bms")));
        assert!(is_bms_file(Path::new("/path/to/song.BME")));
        assert!(is_bms_file(Path::new("/path/to/song.bml")));
        assert!(is_bms_file(Path::new("/path/to/song.pms")));
        assert!(!is_bms_file(Path::new("/path/to/song.mp3")));
        assert!(!is_bms_file(Path::new("/path/to/song.txt")));
    }

    #[test]
    fn test_decode_bms_content_utf8() {
        let bytes = "テスト".as_bytes();
        let decoded = decode_bms_content(bytes);
        assert_eq!(decoded, "テスト");
    }
}

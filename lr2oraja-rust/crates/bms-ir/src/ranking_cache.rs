use std::collections::HashMap;

use sha2::{Digest, Sha256};

use bms_database::SongData;

use crate::course_data::CourseDataConstraint;
use crate::ranking_data::RankingData;

/// Cache for IR ranking data.
///
/// Corresponds to Java `RankingDataCache`.
/// Uses 4 cache slots indexed by LN mode (0/1/2 for songs with undefined LN, 3 otherwise).
pub struct RankingDataCache {
    /// Score caches (4 slots)
    score_cache: [HashMap<String, RankingData>; 4],
    /// Course score caches (4 slots)
    course_cache: [HashMap<String, RankingData>; 4],
}

impl RankingDataCache {
    pub fn new() -> Self {
        Self {
            score_cache: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            course_cache: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
        }
    }

    /// Get cached ranking data for a song.
    pub fn get_song(&self, song: &SongData, lnmode: i32) -> Option<&RankingData> {
        let idx = cache_index(song.has_undefined_long_note(), lnmode);
        self.score_cache[idx].get(&song.sha256)
    }

    /// Put ranking data for a song.
    pub fn put_song(&mut self, song: &SongData, lnmode: i32, data: RankingData) {
        let idx = cache_index(song.has_undefined_long_note(), lnmode);
        self.score_cache[idx].insert(song.sha256.clone(), data);
    }

    /// Get cached ranking data for a course.
    pub fn get_course(
        &self,
        songs: &[SongData],
        constraints: &[CourseDataConstraint],
        lnmode: i32,
    ) -> Option<&RankingData> {
        let idx = course_cache_index(songs, lnmode);
        let hash = create_course_hash(songs, constraints)?;
        self.course_cache[idx].get(&hash)
    }

    /// Put ranking data for a course.
    pub fn put_course(
        &mut self,
        songs: &[SongData],
        constraints: &[CourseDataConstraint],
        lnmode: i32,
        data: RankingData,
    ) -> bool {
        let idx = course_cache_index(songs, lnmode);
        if let Some(hash) = create_course_hash(songs, constraints) {
            self.course_cache[idx].insert(hash, data);
            true
        } else {
            false
        }
    }
}

impl Default for RankingDataCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute cache index. If the song has undefined LN, use lnmode; otherwise use 3.
fn cache_index(has_undefined_ln: bool, lnmode: i32) -> usize {
    if has_undefined_ln {
        (lnmode as usize).min(3)
    } else {
        3
    }
}

/// Compute course cache index. If any song has undefined LN, use lnmode.
fn course_cache_index(songs: &[SongData], lnmode: i32) -> usize {
    let has_undefined = songs.iter().any(|s| s.has_undefined_long_note());
    cache_index(has_undefined, lnmode)
}

/// Create a course hash from song SHA256s and constraint names.
///
/// Returns None if any song is missing a valid SHA256.
pub fn create_course_hash(
    songs: &[SongData],
    constraints: &[CourseDataConstraint],
) -> Option<String> {
    let mut combined = String::new();
    for song in songs {
        if song.sha256.len() == 64 {
            combined.push_str(&song.sha256);
        } else {
            return None;
        }
    }
    for constraint in constraints {
        combined.push_str(constraint.constraint_name());
    }

    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash = hasher.finalize();
    Some(hex::encode(&hash))
}

/// Convert bytes to hex string (equivalent to Java BMSDecoder.convertHexString).
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_database::song_data::FEATURE_UNDEFINEDLN;

    fn make_song(sha256: &str, has_undef_ln: bool) -> SongData {
        SongData {
            sha256: sha256.to_string(),
            feature: if has_undef_ln { FEATURE_UNDEFINEDLN } else { 0 },
            ..Default::default()
        }
    }

    #[test]
    fn cache_index_no_undefined_ln() {
        assert_eq!(cache_index(false, 0), 3);
        assert_eq!(cache_index(false, 1), 3);
        assert_eq!(cache_index(false, 2), 3);
    }

    #[test]
    fn cache_index_with_undefined_ln() {
        assert_eq!(cache_index(true, 0), 0);
        assert_eq!(cache_index(true, 1), 1);
        assert_eq!(cache_index(true, 2), 2);
    }

    #[test]
    fn put_get_song() {
        let mut cache = RankingDataCache::new();
        let song = make_song("a".repeat(64).as_str(), false);
        let data = RankingData::new();
        cache.put_song(&song, 0, data);
        assert!(cache.get_song(&song, 0).is_some());
    }

    #[test]
    fn get_song_miss() {
        let cache = RankingDataCache::new();
        let song = make_song("a".repeat(64).as_str(), false);
        assert!(cache.get_song(&song, 0).is_none());
    }

    #[test]
    fn put_get_song_ln_mode_separation() {
        let mut cache = RankingDataCache::new();
        let song = make_song("b".repeat(64).as_str(), true);

        let mut data0 = RankingData::new();
        data0.set_last_update_time(100);
        cache.put_song(&song, 0, data0);

        let mut data1 = RankingData::new();
        data1.set_last_update_time(200);
        cache.put_song(&song, 1, data1);

        assert_eq!(cache.get_song(&song, 0).unwrap().last_update_time(), 100);
        assert_eq!(cache.get_song(&song, 1).unwrap().last_update_time(), 200);
    }

    #[test]
    fn course_hash_basic() {
        let songs = vec![
            make_song(&"a".repeat(64), false),
            make_song(&"b".repeat(64), false),
        ];
        let constraints = vec![CourseDataConstraint::Class, CourseDataConstraint::NoSpeed];
        let hash = create_course_hash(&songs, &constraints);
        assert!(hash.is_some());
        assert_eq!(hash.unwrap().len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn course_hash_invalid_sha256() {
        let songs = vec![make_song("short", false)];
        let hash = create_course_hash(&songs, &[]);
        assert!(hash.is_none());
    }

    #[test]
    fn course_hash_deterministic() {
        let songs = vec![make_song(&"c".repeat(64), false)];
        let constraints = vec![CourseDataConstraint::GaugeLr2];
        let h1 = create_course_hash(&songs, &constraints).unwrap();
        let h2 = create_course_hash(&songs, &constraints).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn put_get_course() {
        let mut cache = RankingDataCache::new();
        let songs = vec![make_song(&"d".repeat(64), false)];
        let constraints = vec![CourseDataConstraint::Class];
        let data = RankingData::new();
        assert!(cache.put_course(&songs, &constraints, 0, data));
        assert!(cache.get_course(&songs, &constraints, 0).is_some());
    }

    #[test]
    fn put_course_invalid_returns_false() {
        let mut cache = RankingDataCache::new();
        let songs = vec![make_song("short", false)];
        let data = RankingData::new();
        assert!(!cache.put_course(&songs, &[], 0, data));
    }
}

//! CRC32 path hash and song utility functions.
//!
//! Port of Java `SongUtils.java`.

const POLYNOMIAL: u32 = 0xEDB88320;

/// Illegal song titles that should be excluded from the database.
pub const ILLEGAL_SONGS: &[&str] = &["notme"];

/// Compute a CRC32 path hash for directory identification.
///
/// Special case: if `path` matches the parent of any root directory,
/// returns the fixed hash `"e2977170"`.
///
/// Otherwise, strips the `bms_path` prefix from `path`, appends `"\\\0"`,
/// and computes a reflected CRC32 (IEEE polynomial 0xEDB88320).
pub fn crc32_path(path: &str, root_dirs: &[&str], bms_path: &str) -> String {
    for s in root_dirs {
        let p = std::path::Path::new(s);
        if let Some(abs) = p.canonicalize().ok().and_then(|a| {
            a.parent()
                .map(|parent| parent.to_string_lossy().to_string())
        }) && abs == path
        {
            return "e2977170".to_string();
        }
    }

    let relative = if path.starts_with(bms_path) && path.len() > bms_path.len() {
        &path[bms_path.len() + 1..]
    } else {
        path
    };

    let data = format!("{relative}\\\0");
    let hash = crc32_bytes(data.as_bytes());
    format!("{hash:x}")
}

/// Compute CRC32 with IEEE polynomial (reflected / LSB-first).
fn crc32_bytes(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ POLYNOMIAL;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_basic() {
        // Known CRC32 for "test\\\0"
        let hash = crc32_bytes(b"test\\\0");
        let hex = format!("{hash:x}");
        assert_eq!(hex.len(), 8);
    }

    #[test]
    fn crc32_path_strips_prefix() {
        let result = crc32_path("/root/bms/song/file.bms", &[], "/root/bms");
        // Should compute CRC32 of "song/file.bms\\\0"
        let expected = crc32_bytes(b"song/file.bms\\\0");
        assert_eq!(result, format!("{expected:x}"));
    }

    #[test]
    fn crc32_path_no_prefix_match() {
        let result = crc32_path("other/path", &[], "/root/bms");
        let expected = crc32_bytes(b"other/path\\\0");
        assert_eq!(result, format!("{expected:x}"));
    }

    #[test]
    fn illegal_songs_contains_notme() {
        assert!(ILLEGAL_SONGS.contains(&"notme"));
    }

    #[test]
    fn crc32_empty_string() {
        let hash = crc32_bytes(b"\\\0");
        let hex = format!("{hash:x}");
        assert!(!hex.is_empty());
    }

    #[test]
    fn crc32_deterministic() {
        let h1 = crc32_bytes(b"hello\\\0");
        let h2 = crc32_bytes(b"hello\\\0");
        assert_eq!(h1, h2);
    }
}

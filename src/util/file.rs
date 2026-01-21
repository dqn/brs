use std::path::{Path, PathBuf};

/// Try different extensions and case variations to find an existing file.
///
/// Returns the first matching file path, or None if no match is found.
/// This function tries:
/// 1. Original filename
/// 2. Lowercase filename
/// 3. Different extensions (with original case)
/// 4. Different extensions (with lowercase)
pub fn find_file_with_extensions(
    base_path: &Path,
    filename: &str,
    extensions: &[&str],
) -> Option<PathBuf> {
    // Try original filename first
    let file_path = base_path.join(filename);
    if file_path.exists() {
        return Some(file_path);
    }

    // Try lowercase filename
    let lower = filename.to_lowercase();
    let lower_path = base_path.join(&lower);
    if lower_path.exists() {
        return Some(lower_path);
    }

    // Try different extensions
    let stem = Path::new(filename).file_stem()?.to_str()?;
    for ext in extensions {
        // Original case with different extension
        let alt_filename = format!("{}.{}", stem, ext);
        let alt_path = base_path.join(&alt_filename);
        if alt_path.exists() {
            return Some(alt_path);
        }

        // Lowercase with different extension
        let alt_lower = alt_filename.to_lowercase();
        let alt_lower_path = base_path.join(&alt_lower);
        if alt_lower_path.exists() {
            return Some(alt_lower_path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_file_exact_match() {
        let dir = tempdir().expect("failed to create temp directory");
        let file_path = dir.path().join("test.wav");
        fs::write(&file_path, "").expect("failed to create test file");

        let result = find_file_with_extensions(dir.path(), "test.wav", &["ogg", "mp3"]);
        assert_eq!(result, Some(file_path));
    }

    #[test]
    fn test_find_file_lowercase_fallback() {
        let dir = tempdir().expect("failed to create temp directory");
        let file_path = dir.path().join("test.wav");
        fs::write(&file_path, "").expect("failed to create test file");

        let result = find_file_with_extensions(dir.path(), "TEST.WAV", &["ogg", "mp3"]);
        // On case-insensitive filesystems (macOS, Windows), the returned path
        // might have different casing but points to the same file
        assert!(result.is_some());
        let result_path = result.expect("expected Some but got None");
        assert!(result_path.exists());
        assert_eq!(
            result_path
                .file_name()
                .expect("expected file name")
                .to_str()
                .expect("expected valid UTF-8")
                .to_lowercase(),
            "test.wav"
        );
    }

    #[test]
    fn test_find_file_extension_fallback() {
        let dir = tempdir().expect("failed to create temp directory");
        let file_path = dir.path().join("test.ogg");
        fs::write(&file_path, "").expect("failed to create test file");

        let result = find_file_with_extensions(dir.path(), "test.wav", &["ogg", "mp3"]);
        assert_eq!(result, Some(file_path));
    }

    #[test]
    fn test_find_file_not_found() {
        let dir = tempdir().expect("failed to create temp directory");

        let result = find_file_with_extensions(dir.path(), "nonexistent.wav", &["ogg", "mp3"]);
        assert_eq!(result, None);
    }
}

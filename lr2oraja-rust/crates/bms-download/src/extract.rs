// Archive extraction
//
// Provides trait-based extraction with tar.gz implementation.
// 7z support is omitted for now (would require sevenz-rust).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, bail};
use flate2::read::GzDecoder;

/// Extract a .tar.gz archive to the destination directory.
pub fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    let file =
        File::open(archive_path).with_context(|| format!("failed to open {:?}", archive_path))?;
    let reader = BufReader::new(file);
    let decoder = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest_dir)
        .with_context(|| format!("failed to extract {:?} to {:?}", archive_path, dest_dir))?;
    Ok(())
}

/// Detect archive format by extension and extract.
///
/// Currently supports:
/// - `.tar.gz`, `.tgz` — tar + gzip
///
/// Returns an error for unsupported formats.
pub fn detect_and_extract(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    let name = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest_dir)
    } else {
        bail!(
            "unsupported archive format: {:?} (supported: .tar.gz, .tgz)",
            archive_path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    /// Create a minimal tar.gz archive containing a single file.
    fn create_test_tar_gz(
        dir: &Path,
        archive_name: &str,
        file_name: &str,
        content: &[u8],
    ) -> std::path::PathBuf {
        let archive_path = dir.join(archive_name);

        let file = File::create(&archive_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);

        let mut header = tar::Header::new_gnu();
        header.set_path(file_name).unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, content).unwrap();
        builder.finish().unwrap();

        archive_path
    }

    #[test]
    fn test_extract_tar_gz() {
        let tmp = tempfile::tempdir().unwrap();
        let archive_dir = tmp.path().join("archives");
        let extract_dir = tmp.path().join("output");
        std::fs::create_dir_all(&archive_dir).unwrap();
        std::fs::create_dir_all(&extract_dir).unwrap();

        let archive =
            create_test_tar_gz(&archive_dir, "test.tar.gz", "hello.txt", b"Hello, World!");

        extract_tar_gz(&archive, &extract_dir).unwrap();

        let extracted = extract_dir.join("hello.txt");
        assert!(extracted.exists());
        assert_eq!(
            std::fs::read_to_string(&extracted).unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn test_extract_tar_gz_with_subdirectory() {
        let tmp = tempfile::tempdir().unwrap();
        let archive_dir = tmp.path().join("archives");
        let extract_dir = tmp.path().join("output");
        std::fs::create_dir_all(&archive_dir).unwrap();
        std::fs::create_dir_all(&extract_dir).unwrap();

        let archive_path = archive_dir.join("nested.tar.gz");
        let file = File::create(&archive_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);

        // Add a directory entry
        let mut dir_header = tar::Header::new_gnu();
        dir_header.set_path("subdir/").unwrap();
        dir_header.set_size(0);
        dir_header.set_mode(0o755);
        dir_header.set_entry_type(tar::EntryType::Directory);
        dir_header.set_cksum();
        builder.append(&dir_header, &[] as &[u8]).unwrap();

        // Add a file in the subdirectory
        let content = b"nested content";
        let mut file_header = tar::Header::new_gnu();
        file_header.set_path("subdir/file.bms").unwrap();
        file_header.set_size(content.len() as u64);
        file_header.set_mode(0o644);
        file_header.set_cksum();
        builder.append(&file_header, &content[..]).unwrap();

        // Finish the tar archive and flush the gzip encoder
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();

        extract_tar_gz(&archive_path, &extract_dir).unwrap();

        let extracted = extract_dir.join("subdir/file.bms");
        assert!(extracted.exists());
        assert_eq!(
            std::fs::read_to_string(&extracted).unwrap(),
            "nested content"
        );
    }

    #[test]
    fn test_detect_and_extract_tar_gz() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = create_test_tar_gz(tmp.path(), "test.tar.gz", "data.txt", b"test data");

        let extract_dir = tmp.path().join("out");
        std::fs::create_dir_all(&extract_dir).unwrap();

        detect_and_extract(&archive, &extract_dir).unwrap();
        assert!(extract_dir.join("data.txt").exists());
    }

    #[test]
    fn test_detect_and_extract_tgz() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = create_test_tar_gz(tmp.path(), "test.tgz", "data.txt", b"tgz data");

        let extract_dir = tmp.path().join("out");
        std::fs::create_dir_all(&extract_dir).unwrap();

        detect_and_extract(&archive, &extract_dir).unwrap();
        assert!(extract_dir.join("data.txt").exists());
    }

    #[test]
    fn test_detect_and_extract_unsupported() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.7z");
        File::create(&path).unwrap().write_all(b"fake").unwrap();

        let extract_dir = tmp.path().join("out");
        std::fs::create_dir_all(&extract_dir).unwrap();

        let result = detect_and_extract(&path, &extract_dir);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unsupported archive format"));
    }

    #[test]
    fn test_extract_nonexistent_file() {
        let tmp = tempfile::tempdir().unwrap();
        let result = extract_tar_gz(&tmp.path().join("nonexistent.tar.gz"), tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_invalid_tar_gz() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("invalid.tar.gz");
        File::create(&path)
            .unwrap()
            .write_all(b"not a real archive")
            .unwrap();

        let extract_dir = tmp.path().join("out");
        std::fs::create_dir_all(&extract_dir).unwrap();

        let result = extract_tar_gz(&path, &extract_dir);
        assert!(result.is_err());
    }
}

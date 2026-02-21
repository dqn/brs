use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::{error, warn};

/// Corresponds to Java's `bms.player.beatoraja.system.RobustFile`.
/// Provides robust file load/write with backup and atomic rename.
pub struct RobustFile;

impl RobustFile {
    /// Corresponds to Java's `backupPath(Path original)`.
    /// Returns sibling path with ".bak" appended to the filename.
    fn backup_path(original: &Path) -> PathBuf {
        let file_name = original
            .file_name()
            .map(|n| {
                let mut s = n.to_os_string();
                s.push(".bak");
                s
            })
            .unwrap_or_default();
        original.with_file_name(file_name)
    }

    /// Corresponds to Java's `temporaryPath(Path original)`.
    /// Returns sibling path with ".tmp" appended to the filename.
    fn temporary_path(original: &Path) -> PathBuf {
        let file_name = original
            .file_name()
            .map(|n| {
                let mut s = n.to_os_string();
                s.push(".tmp");
                s
            })
            .unwrap_or_default();
        original.with_file_name(file_name)
    }

    /// Corresponds to Java's `load(Path file, Parser<T> parser)`.
    /// Reads and parses the file. In case of failure, falls back to trying the backup file.
    pub fn load<T, F>(file: &Path, parser: F) -> Result<T>
    where
        F: Fn(&[u8]) -> Result<T>,
    {
        // try {
        //     data = Files.readAllBytes(file);
        //     return parser.apply(data);
        // }
        match fs::read(file).and_then(|data| {
            parser(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }) {
            Ok(result) => Ok(result),
            Err(e) => {
                // catch (IOException e) or catch (ParseException e)
                // could not read the original file or parsing failed - try the backup
                error!("{}", e);
                Self::load_backup(file, parser)
            }
        }
    }

    /// Corresponds to Java's `loadBackup(Path original, Parser<T> parser)`.
    /// Reads and parses the backup file (.bak).
    pub fn load_backup<T, F>(original: &Path, parser: F) -> Result<T>
    where
        F: Fn(&[u8]) -> Result<T>,
    {
        let file = Self::backup_path(original);
        // if (!Files.isRegularFile(file)) {
        //     throw new IOException("File load failed: No backup file. \nPath: " + original);
        // }
        if !file.is_file() {
            anyhow::bail!(
                "File load failed: No backup file. \nPath: {}",
                original.display()
            );
        }

        // try {
        //     data = Files.readAllBytes(file);
        //     return parser.apply(data);
        // }
        let data = fs::read(&file).with_context(|| {
            format!(
                "File load failed.\nPath: {}\nReason: IOException",
                original.display()
            )
        })?;

        parser(&data).with_context(|| {
            format!(
                "File load failed.\nPath: {}\nReason: ParseException",
                original.display()
            )
        })
    }

    /// Corresponds to Java's `write(Path file, byte[] data)`.
    /// Writes backup, then temporary file, then atomic rename temporary to original.
    pub fn write(file: &Path, data: &[u8]) -> Result<()> {
        // write backup & fsync
        // write temporary file & fsync
        // rename temporary to original

        // each of these writes can individually throw, aborting the operation
        // we don't perform any retries, since the error might be persistent
        Self::write_file(&Self::backup_path(file), data)?;
        Self::write_file(&Self::temporary_path(file), data)?;
        // we only perform the final rename if both writes completed successfully

        // Note that, even though we request an atomic rename, this is not actually an atomic
        // operation with respect to system crashes, and not at all on certain filesystems.

        // That's the reason for the double-write scheme, where we first create
        // a backup, then a temporary copy and rename the temporary into the original.
        // Even if replacing the original with the temporary fails, we should
        // still be able to read the new data from its backup; if creating
        // the backup fails, the original file will remain untouched.

        let tmp_path = Self::temporary_path(file);
        // try {
        //     Files.move(temporaryPath(file), file, REPLACE_EXISTING, ATOMIC_MOVE);
        // }
        // catch (AtomicMoveNotSupportedException e) {
        //     logger.warn("RobustFile.write: Could not perform an atomic move to {}", file);
        //     Files.move(temporaryPath(file), file, REPLACE_EXISTING);
        // }
        match fs::rename(&tmp_path, file) {
            Ok(()) => {}
            Err(_e) => {
                warn!(
                    "RobustFile.write: Could not perform an atomic move to {}",
                    file.display()
                );
                // Fallback: copy then remove (non-atomic)
                fs::copy(&tmp_path, file).with_context(|| {
                    format!("Failed to copy temporary file to {}", file.display())
                })?;
                // Best-effort remove of temp file
                let _ = fs::remove_file(&tmp_path);
            }
        }

        // This approach does nothing whatsoever to protect against in-memory data corruption,
        // or erroneous writes; which means this operation can complete successfully,
        // but as a result overwrites the config file with unusable data.
        // Checksumming each file and verifying after the write would be an expensive operation,
        // and possibly unproductive on systems where we can't ensure that the data we read
        // back actually comes from the device, rather than from cache.

        // In the case that both the original and backup files become damaged
        // and cannot be loaded, we might want to consider entirely preventing
        // the game from launching and inadvertently resetting the config file
        // to default values, as minor corruption might still be manually fixable.

        Ok(())
    }

    /// Corresponds to Java's `writeFile(Path file, byte[] data)`.
    /// Opens file, writes data, and fsyncs.
    pub fn write_file(file: &Path, data: &[u8]) -> Result<()> {
        // try (FileChannel outChannel = FileChannel.open(file, CREATE, TRUNCATE_EXISTING, WRITE)) {
        //     outChannel.write(ByteBuffer.wrap(data));
        //     outChannel.force(true);
        // }
        let mut f = fs::File::create(file)
            .with_context(|| format!("Failed to create file {}", file.display()))?;
        f.write_all(data)
            .with_context(|| format!("Failed to write data to {}", file.display()))?;
        // force corresponds to:
        // on linux, fsync(fd)
        // on macOS, fcntl(fd, F_FULLFSYNC)
        // on windows, FlushFileBuffers(hFile)
        //
        // all of these should request that the data is
        // actually written to device before proceeding
        f.sync_all()
            .with_context(|| format!("Failed to fsync file {}", file.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_path() {
        let p = Path::new("/foo/bar/config.json");
        assert_eq!(
            RobustFile::backup_path(p),
            PathBuf::from("/foo/bar/config.json.bak")
        );
    }

    #[test]
    fn test_temporary_path() {
        let p = Path::new("/foo/bar/config.json");
        assert_eq!(
            RobustFile::temporary_path(p),
            PathBuf::from("/foo/bar/config.json.tmp")
        );
    }

    #[test]
    fn test_write_and_load() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.dat");
        let data = b"hello world";

        // write
        RobustFile::write(&file, data).unwrap();

        // original file should exist with correct content
        assert_eq!(fs::read(&file).unwrap(), data);

        // backup should also exist
        let bak = RobustFile::backup_path(&file);
        assert!(bak.is_file());
        assert_eq!(fs::read(&bak).unwrap(), data);

        // load should parse successfully
        let result: String = RobustFile::load(&file, |d| {
            Ok(String::from_utf8(d.to_vec()).map_err(|e| anyhow::anyhow!(e))?)
        })
        .unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_load_falls_back_to_backup() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.dat");
        let bak = dir.path().join("test.dat.bak");

        // Only create backup, not original
        fs::write(&bak, b"backup data").unwrap();

        let result: String = RobustFile::load(&file, |d| {
            Ok(String::from_utf8(d.to_vec()).map_err(|e| anyhow::anyhow!(e))?)
        })
        .unwrap();
        assert_eq!(result, "backup data");
    }

    #[test]
    fn test_load_backup_no_backup_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("nonexistent.dat");

        let result: Result<String> = RobustFile::load_backup(&file, |d| {
            Ok(String::from_utf8(d.to_vec()).map_err(|e| anyhow::anyhow!(e))?)
        });
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No backup file"));
    }

    #[test]
    fn test_load_parse_failure_falls_back_to_backup() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.dat");
        let bak = dir.path().join("test.dat.bak");

        // Original has bad data, backup has good data
        fs::write(&file, b"bad data").unwrap();
        fs::write(&bak, b"42").unwrap();

        let result: i32 = RobustFile::load(&file, |d| {
            let s = std::str::from_utf8(d)?;
            let n: i32 = s
                .parse()
                .map_err(|e: std::num::ParseIntError| anyhow::anyhow!(e))?;
            Ok(n)
        })
        .unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_write_file_fsync() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("sync_test.dat");
        let data = b"sync me";

        RobustFile::write_file(&file, data).unwrap();
        assert_eq!(fs::read(&file).unwrap(), data);
    }
}

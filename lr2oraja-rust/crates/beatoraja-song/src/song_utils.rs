use std::path::Path;

const POLYNOMIAL: u32 = 0xEDB88320;

pub fn crc32(path: &str, rootdirs: &[String], bmspath: &str) -> String {
    let mut path = path.to_string();

    for s in rootdirs {
        if let Some(parent) = Path::new(s).parent() {
            if parent.to_string_lossy() == path {
                return "e2977170".to_string();
            }
        }
    }

    if path.starts_with(bmspath) && path.len() > bmspath.len() + 1 {
        path = path[bmspath.len() + 1..].to_string();
    }

    let previous_crc32: u32 = 0;
    let mut crc: u32 = !previous_crc32; // same as previousCrc32 ^ 0xFFFFFFFF

    let bytes_str = format!("{}\\\0", path);
    for b in bytes_str.as_bytes() {
        crc ^= *b as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ POLYNOMIAL;
            } else {
                crc >>= 1;
            }
        }
    }
    format!("{:x}", !crc) // same as crc ^ 0xFFFFFFFF
}

pub static ILLEGAL_SONGS: &[&str] = &["notme"];

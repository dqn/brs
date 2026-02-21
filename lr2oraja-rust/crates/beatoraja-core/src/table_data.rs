use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};

use crate::course_data::CourseData;
use crate::stubs::SongData;
use crate::validatable::{Validatable, remove_invalid_elements_vec};

/// Table data (difficulty table)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TableData {
    pub url: String,
    pub name: String,
    pub tag: String,
    pub folder: Vec<TableFolder>,
    pub course: Vec<CourseData>,
}

impl TableData {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_url_opt(&self) -> Option<&str> {
        if self.url.is_empty() {
            None
        } else {
            Some(&self.url)
        }
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn get_folder(&self) -> &[TableFolder] {
        &self.folder
    }

    pub fn set_folder(&mut self, folder: Vec<TableFolder>) {
        self.folder = folder;
    }

    pub fn get_course(&self) -> &[CourseData] {
        &self.course
    }

    pub fn set_course(&mut self, course: Vec<CourseData>) {
        self.course = course;
    }

    pub fn shrink(&mut self) {
        for c in &mut self.course {
            c.shrink();
        }
        for tf in &mut self.folder {
            tf.shrink();
        }
    }

    pub fn read_from_path(p: &Path) -> Option<TableData> {
        let path_str = p.to_string_lossy();
        let data: Option<Vec<u8>> = if path_str.ends_with(".bmt") {
            let file = std::fs::File::open(p).ok()?;
            let mut gz = GzDecoder::new(BufReader::new(file));
            let mut buf = Vec::new();
            gz.read_to_end(&mut buf).ok()?;
            Some(buf)
        } else if path_str.ends_with(".json") {
            std::fs::read(p).ok()
        } else {
            None
        };

        if let Some(data) = data {
            let mut td: TableData = serde_json::from_slice(&data).ok()?;
            if td.validate() {
                return Some(td);
            }
        }
        None
    }

    pub fn write_to_path(p: &Path, td: &TableData) {
        let mut td = td.clone();
        td.shrink();
        let path_str = p.to_string_lossy();
        let json = match serde_json::to_string_pretty(&td) {
            Ok(j) => j,
            Err(_) => return,
        };

        if path_str.ends_with(".bmt") {
            if let Ok(file) = std::fs::File::create(p) {
                let mut encoder = GzEncoder::new(BufWriter::new(file), Compression::default());
                let _ = encoder.write_all(json.as_bytes());
                let _ = encoder.finish();
            }
        } else if path_str.ends_with(".json")
            && let Ok(mut file) = std::fs::File::create(p)
        {
            let _ = file.write_all(json.as_bytes());
        }
    }
}

impl Validatable for TableData {
    fn validate(&mut self) -> bool {
        if self.name.is_empty() {
            return false;
        }
        self.folder = remove_invalid_elements_vec(std::mem::take(&mut self.folder));
        self.course = remove_invalid_elements_vec(std::mem::take(&mut self.course));
        self.folder.len() + self.course.len() > 0
    }
}

/// Table folder
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TableFolder {
    pub name: Option<String>,
    pub songs: Vec<SongData>,
}

impl TableFolder {
    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// Returns songs (named `get_song` to match Java API)
    pub fn get_song(&self) -> &[SongData] {
        &self.songs
    }

    pub fn set_song(&mut self, songs: Vec<SongData>) {
        self.songs = songs;
    }

    pub fn shrink(&mut self) {
        for song in &mut self.songs {
            song.shrink();
        }
    }
}

impl Validatable for TableFolder {
    fn validate(&mut self) -> bool {
        self.songs.retain_mut(|s| s.validate());
        self.name.as_ref().is_some_and(|n| !n.is_empty()) && !self.songs.is_empty()
    }
}

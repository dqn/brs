// Bar navigation — folder enter/leave, load, and search methods for BarManager.

use std::collections::BTreeMap;
use std::path::Path;

use bms_database::{CourseData, CourseDataAccessor, SongDatabase, TableData};

use super::BarManager;
use super::bar_types::Bar;

/// JSON structure for command folder definitions (folder/default.json).
///
/// Matches Java `BarManager.CommandFolder` with nested folder/SQL support.
#[derive(Debug, Clone, serde::Deserialize)]
struct CommandFolder {
    name: String,
    #[serde(default)]
    sql: String,
    #[serde(default)]
    folder: Vec<CommandFolder>,
    /// Whether to show all songs (Java parity field, not yet used in Rust).
    #[serde(default)]
    #[allow(dead_code)]
    showall: bool,
}

impl BarManager {
    /// Load root bar list from the database.
    ///
    /// Groups songs by their `folder` field (CRC) into folder bars,
    /// matching the Java `BarManager.updateBar()` root structure.
    pub fn load_root(&mut self, song_db: &SongDatabase) {
        let songs = song_db.get_all_song_datas().unwrap_or_default();

        // Group songs by folder CRC, preserving insertion order via BTreeMap (sorted by CRC)
        let mut folder_groups: BTreeMap<String, String> = BTreeMap::new();
        for song in &songs {
            if !folder_groups.contains_key(&song.folder) {
                // Derive folder name from the first song's parent directory
                let name = Path::new(&song.path)
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| song.folder.clone());
                folder_groups.insert(song.folder.clone(), name);
            }
        }

        // Build folder bars (each folder expands to its songs via enter_folder)
        self.bars = folder_groups
            .into_iter()
            .map(|(folder_crc, name)| Bar::Folder {
                name,
                path: folder_crc,
            })
            .collect();

        self.cursor = 0;
        self.folder_stack.clear();
    }

    /// Enter the currently selected folder.
    /// Pushes current bars and cursor onto the stack, loads folder contents.
    pub fn enter_folder(&mut self, song_db: &SongDatabase) {
        match self.bars.get(self.cursor) {
            Some(Bar::Folder { path, .. }) => {
                let folder_path = path.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));

                let songs = song_db
                    .get_song_datas("folder", &folder_path)
                    .unwrap_or_default();
                self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
                self.cursor = 0;
            }
            Some(Bar::TableRoot {
                folders, courses, ..
            }) => {
                let folders = folders.clone();
                let courses = courses.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));

                let mut new_bars: Vec<Bar> = Vec::new();
                // Add level folders as HashFolder bars
                for folder in &folders {
                    let hashes: Vec<String> = folder
                        .songs
                        .iter()
                        .map(|s| {
                            if !s.sha256.is_empty() {
                                s.sha256.clone()
                            } else {
                                s.md5.clone()
                            }
                        })
                        .collect();
                    new_bars.push(Bar::HashFolder {
                        name: folder.name.clone(),
                        hashes,
                    });
                }
                // Add courses
                for course in &courses {
                    new_bars.push(Bar::Course(Box::new(course.clone())));
                }
                self.bars = new_bars;
                self.cursor = 0;
            }
            Some(Bar::HashFolder { hashes, .. }) => {
                let hashes = hashes.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));

                let hash_refs: Vec<&str> = hashes.iter().map(String::as_str).collect();
                let songs = song_db
                    .get_song_datas_by_hashes(&hash_refs)
                    .unwrap_or_default();
                self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
                self.cursor = 0;
            }
            Some(Bar::Container { children, .. }) => {
                let children = children.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));
                self.bars = children;
                self.cursor = 0;
            }
            Some(Bar::SameFolder { folder_crc, .. }) => {
                let crc = folder_crc.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));

                let songs = song_db.get_song_datas("folder", &crc).unwrap_or_default();
                self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
                self.cursor = 0;
            }
            Some(Bar::SearchWord { query }) => {
                let query = query.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));

                let songs = song_db.get_song_datas_by_text(&query).unwrap_or_default();
                self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
                self.cursor = 0;
            }
            Some(Bar::Command { sql, .. }) => {
                let sql = sql.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));

                // Execute custom SQL query with read-only validation
                let songs = song_db.get_song_datas_by_sql(&sql).unwrap_or_default();
                self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
                self.cursor = 0;
            }
            Some(Bar::ContextMenu(cm)) => {
                let items = cm.items.clone();
                let old_bars = std::mem::take(&mut self.bars);
                let old_cursor = self.cursor;
                self.folder_stack.push((old_bars, old_cursor));

                // Expand context menu items as Function bars
                self.bars = items
                    .into_iter()
                    .map(|item| Bar::Function {
                        title: item.label,
                        subtitle: None,
                        display_bar_type: 3,
                        action: item.action,
                        lamp: 0,
                    })
                    .collect();
                self.cursor = 0;
            }
            _ => (),
        }
    }

    /// Leave the current folder, restoring the parent bar list and cursor.
    pub fn leave_folder(&mut self) {
        if let Some((bars, cursor)) = self.folder_stack.pop() {
            self.bars = bars;
            self.cursor = cursor;
        }
    }

    /// Load table data from cache and add TableRoot bars to the root bar list.
    pub fn load_tables(&mut self, tables: &[TableData]) {
        for table in tables {
            self.bars.push(Bar::TableRoot {
                name: table.name.clone(),
                folders: table.folder.clone(),
                courses: table.course.clone(),
            });
        }
    }

    /// Load course data from a directory and add as a "COURSE" TableRoot bar.
    ///
    /// Java parity: `CourseDataAccessor("course").readAll()` → `TableBar(courses)`.
    pub fn load_courses(&mut self, course_dir: &str) {
        match CourseDataAccessor::new(course_dir) {
            Ok(accessor) => match accessor.read_all() {
                Ok(courses) if !courses.is_empty() => {
                    self.bars.push(Bar::TableRoot {
                        name: "COURSE".to_string(),
                        folders: Vec::new(),
                        courses,
                    });
                    tracing::info!(count = self.bars.len(), "Loaded course data");
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Failed to read course data: {e}");
                }
            },
            Err(e) => {
                tracing::warn!("Failed to open course directory: {e}");
            }
        }
    }

    /// Load favorite data from a directory and add as HashFolder bars.
    ///
    /// Java parity: `CourseDataAccessor("favorite").readAll()` → `HashBar[]`.
    pub fn load_favorites(&mut self, favorite_dir: &str) {
        match CourseDataAccessor::new(favorite_dir) {
            Ok(accessor) => match accessor.read_all() {
                Ok(courses) => {
                    for course in &courses {
                        let hashes: Vec<String> = course
                            .hash
                            .iter()
                            .map(|s| {
                                if !s.sha256.is_empty() {
                                    s.sha256.clone()
                                } else {
                                    s.md5.clone()
                                }
                            })
                            .collect();
                        if !hashes.is_empty() {
                            self.bars.push(Bar::HashFolder {
                                name: course.name.clone(),
                                hashes,
                            });
                        }
                    }
                    if !courses.is_empty() {
                        tracing::info!(favorites = courses.len(), "Loaded favorite playlists");
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read favorite data: {e}");
                }
            },
            Err(e) => {
                tracing::warn!("Failed to open favorite directory: {e}");
            }
        }
    }

    /// Load command folders from a JSON file and add as Command/Container bars.
    ///
    /// Java parity: `CommandFolder[]` from `folder/default.json`.
    pub fn load_command_folders(&mut self, json_path: &str) {
        let path = Path::new(json_path);
        if !path.exists() {
            return;
        }
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Vec<CommandFolder>>(&content) {
                Ok(folders) => {
                    for folder in folders {
                        self.bars.push(create_command_bar(folder));
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse command folder JSON: {e}");
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read command folder file: {e}");
            }
        }
    }

    /// Load course data and add them as bars.
    // TODO: integrate with course selection UI — used in tests
    #[allow(dead_code)]
    pub fn add_courses(&mut self, courses: &[CourseData]) {
        for course in courses {
            self.bars.push(Bar::Course(Box::new(course.clone())));
        }
    }

    /// Replace the current folder's bars with new bars (e.g., from IR fetch).
    pub fn replace_current_bars(&mut self, bars: Vec<Bar>) {
        self.bars = bars;
        self.cursor = 0;
    }

    /// Push the current bars onto the folder stack and set new bars.
    ///
    /// Used by leaderboard entry where we don't have a `SongDatabase` reference
    /// but still need the push/pop folder navigation pattern.
    pub fn push_and_set_bars(&mut self, bars: Vec<Bar>) {
        let old_bars = std::mem::take(&mut self.bars);
        let old_cursor = self.cursor;
        self.folder_stack.push((old_bars, old_cursor));
        self.bars = bars;
        self.cursor = 0;
    }

    /// Search for songs matching the query text, pushing the current bar list onto the folder stack.
    pub fn search(&mut self, song_db: &SongDatabase, query: &str) {
        let songs = song_db.get_song_datas_by_text(query).unwrap_or_default();
        // Save current state to folder stack
        let old_bars = std::mem::take(&mut self.bars);
        let old_cursor = self.cursor;
        self.folder_stack.push((old_bars, old_cursor));
        self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
        self.cursor = 0;
    }
}

/// Create a Bar from a CommandFolder definition.
///
/// If the folder has child folders, creates a Container; otherwise creates a Command bar.
/// Matches Java `createCommandBar()`.
fn create_command_bar(folder: CommandFolder) -> Bar {
    if !folder.folder.is_empty() {
        // Nested: create Container with child bars
        let children: Vec<Bar> = folder.folder.into_iter().map(create_command_bar).collect();
        Bar::Container {
            name: folder.name,
            children,
        }
    } else if !folder.sql.is_empty() {
        // Leaf: create Command bar with SQL query
        Bar::Command {
            name: folder.name,
            sql: format!("SELECT * FROM song WHERE {}", folder.sql),
        }
    } else {
        // Empty folder: create Command with no-result query
        Bar::Command {
            name: folder.name,
            sql: "SELECT * FROM song WHERE 0".to_string(),
        }
    }
}

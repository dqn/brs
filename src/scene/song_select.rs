use std::collections::HashMap;
use std::path::PathBuf;

use macroquad::prelude::*;
use walkdir::WalkDir;

use crate::database::{SavedScore, ScoreRepository, compute_file_hash};
use crate::game::ClearLamp;

use super::{GameplayScene, Scene, SceneTransition, SettingsScene};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    #[default]
    Title,
    Artist,
    Level,
}

impl SortKey {
    fn next(self) -> Self {
        match self {
            SortKey::Title => SortKey::Artist,
            SortKey::Artist => SortKey::Level,
            SortKey::Level => SortKey::Title,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            SortKey::Title => "Title",
            SortKey::Artist => "Artist",
            SortKey::Level => "Level",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SongEntry {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub level: u32,
    pub hash: String,
}

pub struct SongSelectScene {
    all_songs: Vec<SongEntry>,
    filtered_songs: Vec<usize>, // Indices into all_songs
    selected_index: usize,
    scroll_offset: usize,
    visible_count: usize,
    sort_key: SortKey,
    level_filter: Option<u32>, // Filter by minimum level
    scores: HashMap<String, SavedScore>,
}

impl SongSelectScene {
    pub fn new(search_path: Option<&str>) -> Self {
        let all_songs = if let Some(path) = search_path {
            Self::scan_songs(path)
        } else {
            Vec::new()
        };

        let filtered_songs: Vec<usize> = (0..all_songs.len()).collect();

        // Load scores from repository
        let scores = ScoreRepository::new()
            .map(|repo| repo.all().clone())
            .unwrap_or_default();

        let mut scene = Self {
            all_songs,
            filtered_songs,
            selected_index: 0,
            scroll_offset: 0,
            visible_count: 15,
            sort_key: SortKey::Title,
            level_filter: None,
            scores,
        };
        scene.apply_sort();
        scene
    }

    fn get_clear_lamp(&self, hash: &str) -> ClearLamp {
        self.scores
            .get(hash)
            .map(|s| ClearLamp::from_u8(s.clear_lamp))
            .unwrap_or(ClearLamp::NoPlay)
    }

    fn clear_lamp_color(lamp: ClearLamp) -> Color {
        match lamp {
            ClearLamp::NoPlay => Color::new(0.2, 0.2, 0.2, 1.0),
            ClearLamp::Failed => Color::new(0.5, 0.0, 0.0, 1.0),
            ClearLamp::AssistEasy => Color::new(0.6, 0.3, 0.8, 1.0),
            ClearLamp::Easy => Color::new(0.0, 0.8, 0.3, 1.0),
            ClearLamp::Normal => Color::new(0.2, 0.6, 1.0, 1.0),
            ClearLamp::Hard => Color::new(1.0, 0.5, 0.0, 1.0),
            ClearLamp::ExHard => Color::new(1.0, 0.8, 0.0, 1.0),
            ClearLamp::FullCombo => Color::new(1.0, 0.0, 0.5, 1.0),
        }
    }

    fn apply_sort(&mut self) {
        let all_songs = &self.all_songs;
        let sort_key = self.sort_key;

        self.filtered_songs.sort_by(|&a, &b| {
            let song_a = &all_songs[a];
            let song_b = &all_songs[b];
            match sort_key {
                SortKey::Title => song_a.title.cmp(&song_b.title),
                SortKey::Artist => song_a.artist.cmp(&song_b.artist),
                SortKey::Level => song_a.level.cmp(&song_b.level),
            }
        });
    }

    fn apply_filter(&mut self) {
        self.filtered_songs = self
            .all_songs
            .iter()
            .enumerate()
            .filter(|(_, song)| {
                if let Some(min_level) = self.level_filter {
                    song.level >= min_level
                } else {
                    true
                }
            })
            .map(|(i, _)| i)
            .collect();
        self.apply_sort();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn cycle_sort(&mut self) {
        self.sort_key = self.sort_key.next();
        self.apply_sort();
    }

    fn cycle_level_filter(&mut self) {
        self.level_filter = match self.level_filter {
            None => Some(1),
            Some(1) => Some(5),
            Some(5) => Some(10),
            Some(10) => Some(12),
            Some(_) => None,
        };
        self.apply_filter();
    }

    fn get_song(&self, display_index: usize) -> Option<&SongEntry> {
        self.filtered_songs
            .get(display_index)
            .map(|&i| &self.all_songs[i])
    }

    fn scan_songs(path: &str) -> Vec<SongEntry> {
        let mut songs = Vec::new();

        for entry in WalkDir::new(path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                // Support BMS, BME, BML (BMS formats), PMS (Pop'n Music format), and BMSON
                if ext_lower == "bms"
                    || ext_lower == "bme"
                    || ext_lower == "bml"
                    || ext_lower == "pms"
                    || ext_lower == "bmson"
                {
                    if let Some(entry) = Self::parse_header(path.to_path_buf()) {
                        songs.push(entry);
                    }
                }
            }
        }

        songs
    }

    fn parse_header(path: PathBuf) -> Option<SongEntry> {
        // Check if BMSON format
        let is_bmson = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bmson"));

        if is_bmson {
            return Self::parse_bmson_header(path);
        }

        Self::parse_bms_header(path)
    }

    fn parse_bms_header(path: PathBuf) -> Option<SongEntry> {
        let content = std::fs::read_to_string(&path).ok()?;

        let mut title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut artist = String::from("Unknown");
        let mut level = 0u32;

        for line in content.lines() {
            let line = line.trim();
            if let Some(stripped) = line.strip_prefix("#TITLE") {
                title = stripped.trim().to_string();
            } else if let Some(stripped) = line.strip_prefix("#ARTIST") {
                artist = stripped.trim().to_string();
            } else if let Some(stripped) = line.strip_prefix("#PLAYLEVEL") {
                level = stripped.trim().parse().unwrap_or(0);
            }
        }

        // Compute hash for score lookup
        let hash = compute_file_hash(&path).unwrap_or_default();

        Some(SongEntry {
            path,
            title,
            artist,
            level,
            hash,
        })
    }

    fn parse_bmson_header(path: PathBuf) -> Option<SongEntry> {
        let content = std::fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        let info = json.get("info")?;

        let title = info
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            });

        let artist = info
            .get("artist")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::from("Unknown"));

        let level = info.get("level").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        // Compute hash for score lookup
        let hash = compute_file_hash(&path).unwrap_or_default();

        Some(SongEntry {
            path,
            title,
            artist,
            level,
            hash,
        })
    }

    fn update_scroll(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_count {
            self.scroll_offset = self.selected_index - self.visible_count + 1;
        }
    }
}

impl Scene for SongSelectScene {
    fn update(&mut self) -> SceneTransition {
        if self.filtered_songs.is_empty() {
            // Still allow sort/filter changes even with no songs
            if is_key_pressed(KeyCode::Tab) {
                self.cycle_sort();
            }
            if is_key_pressed(KeyCode::F) {
                self.cycle_level_filter();
            }
            if is_key_pressed(KeyCode::Escape) {
                return SceneTransition::Pop;
            }
            return SceneTransition::None;
        }

        if is_key_pressed(KeyCode::Up) && self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll();
        }

        if is_key_pressed(KeyCode::Down) && self.selected_index < self.filtered_songs.len() - 1 {
            self.selected_index += 1;
            self.update_scroll();
        }

        if is_key_pressed(KeyCode::Tab) {
            self.cycle_sort();
        }

        if is_key_pressed(KeyCode::F) {
            self.cycle_level_filter();
        }

        if is_key_pressed(KeyCode::Enter) {
            if let Some(song) = self.get_song(self.selected_index) {
                let path = song.path.to_string_lossy().to_string();
                return SceneTransition::Push(Box::new(GameplayScene::new(path)));
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }

        if is_key_pressed(KeyCode::S) {
            return SceneTransition::Push(Box::new(SettingsScene::new()));
        }

        SceneTransition::None
    }

    fn draw(&self) {
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));

        draw_text("SONG SELECT", 20.0, 40.0, 32.0, WHITE);

        // Show filter/sort status
        let filter_text = match self.level_filter {
            None => "All".to_string(),
            Some(lvl) => format!("☆{}+", lvl),
        };
        draw_text(
            &format!(
                "{} songs | Sort: {} | Filter: {}",
                self.filtered_songs.len(),
                self.sort_key.name(),
                filter_text
            ),
            20.0,
            70.0,
            18.0,
            GRAY,
        );

        if self.filtered_songs.is_empty() {
            draw_text(
                "No songs match the current filter.",
                20.0,
                screen_height() / 2.0,
                24.0,
                YELLOW,
            );
            return;
        }

        let start_y = 100.0;
        let item_height = 35.0;

        for (display_idx, &song_idx) in self
            .filtered_songs
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.visible_count)
        {
            let song = &self.all_songs[song_idx];
            let y = start_y + (display_idx - self.scroll_offset) as f32 * item_height;
            let is_selected = display_idx == self.selected_index;

            if is_selected {
                draw_rectangle(
                    10.0,
                    y - 5.0,
                    screen_width() - 20.0,
                    item_height,
                    Color::new(0.2, 0.3, 0.5, 1.0),
                );
            }

            let color = if is_selected { YELLOW } else { WHITE };

            // Clear lamp indicator
            let clear_lamp = self.get_clear_lamp(&song.hash);
            let lamp_color = Self::clear_lamp_color(clear_lamp);
            draw_rectangle(15.0, y, 6.0, item_height - 10.0, lamp_color);

            // Level display
            if song.level > 0 {
                draw_text(&format!("☆{}", song.level), 30.0, y + 18.0, 18.0, SKYBLUE);
                draw_text(&song.title, 80.0, y + 18.0, 22.0, color);
            } else {
                draw_text(&song.title, 30.0, y + 18.0, 22.0, color);
            }
            draw_text(&song.artist, 80.0, y + 32.0, 14.0, GRAY);
        }

        draw_text(
            "[Up/Down] Select | [Enter] Play | [Tab] Sort | [F] Filter | [S] Settings | [Esc] Quit",
            20.0,
            screen_height() - 20.0,
            16.0,
            GRAY,
        );
    }
}

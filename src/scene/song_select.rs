use std::path::PathBuf;

use macroquad::prelude::*;
use walkdir::WalkDir;

use super::{GameplayScene, Scene, SceneTransition};

#[derive(Debug, Clone)]
pub struct SongEntry {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
}

pub struct SongSelectScene {
    songs: Vec<SongEntry>,
    selected_index: usize,
    scroll_offset: usize,
    visible_count: usize,
}

impl SongSelectScene {
    pub fn new(search_path: Option<&str>) -> Self {
        let songs = if let Some(path) = search_path {
            Self::scan_songs(path)
        } else {
            Vec::new()
        };

        Self {
            songs,
            selected_index: 0,
            scroll_offset: 0,
            visible_count: 15,
        }
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
                if ext_lower == "bms" || ext_lower == "bme" || ext_lower == "bml" {
                    if let Some(entry) = Self::parse_header(path.to_path_buf()) {
                        songs.push(entry);
                    }
                }
            }
        }

        songs.sort_by(|a, b| a.title.cmp(&b.title));
        songs
    }

    fn parse_header(path: PathBuf) -> Option<SongEntry> {
        let content = std::fs::read_to_string(&path).ok()?;

        let mut title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut artist = String::from("Unknown");

        for line in content.lines() {
            let line = line.trim();
            if let Some(stripped) = line.strip_prefix("#TITLE") {
                title = stripped.trim().to_string();
            } else if let Some(stripped) = line.strip_prefix("#ARTIST") {
                artist = stripped.trim().to_string();
            }
            if !title.is_empty() && artist != "Unknown" {
                break;
            }
        }

        Some(SongEntry {
            path,
            title,
            artist,
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
        if self.songs.is_empty() {
            return SceneTransition::None;
        }

        if is_key_pressed(KeyCode::Up) && self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll();
        }

        if is_key_pressed(KeyCode::Down) && self.selected_index < self.songs.len() - 1 {
            self.selected_index += 1;
            self.update_scroll();
        }

        if is_key_pressed(KeyCode::Enter) {
            let song = &self.songs[self.selected_index];
            let path = song.path.to_string_lossy().to_string();
            return SceneTransition::Push(Box::new(GameplayScene::new(path)));
        }

        if is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }

        SceneTransition::None
    }

    fn draw(&self) {
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));

        draw_text("SONG SELECT", 20.0, 40.0, 32.0, WHITE);
        draw_text(
            &format!("{} songs found", self.songs.len()),
            20.0,
            70.0,
            20.0,
            GRAY,
        );

        if self.songs.is_empty() {
            draw_text(
                "No songs found. Pass a folder path as argument.",
                20.0,
                screen_height() / 2.0,
                24.0,
                YELLOW,
            );
            return;
        }

        let start_y = 100.0;
        let item_height = 35.0;

        for (i, song) in self
            .songs
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.visible_count)
        {
            let y = start_y + (i - self.scroll_offset) as f32 * item_height;
            let is_selected = i == self.selected_index;

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
            draw_text(&song.title, 30.0, y + 18.0, 22.0, color);
            draw_text(&song.artist, 30.0, y + 32.0, 14.0, GRAY);
        }

        draw_text(
            "[Up/Down] Select | [Enter] Play | [Esc] Quit",
            20.0,
            screen_height() - 20.0,
            16.0,
            GRAY,
        );
    }
}

use crate::config::AppConfig;
use crate::input::{ConfigHotkey, HotkeyConfig, InputManager};
use crate::skin::path as skin_path;
use macroquad::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

/// Result of the library config screen.
/// ライブラリ設定画面の結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryConfigResult {
    None,
    Back { rescan: bool },
}

#[derive(Debug, Clone)]
struct SkinOption {
    label: String,
    path: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum FolderEditMode {
    Add,
    Edit(usize),
}

#[derive(Debug, Clone, Copy)]
enum LibraryItem {
    Folder(usize),
    AddFolder,
    AutoScan,
    Skin,
    ScanNow,
    SaveAndBack,
}

/// Library and skin configuration screen.
/// ライブラリとスキンの設定画面。
pub struct LibraryConfigScreen {
    config: AppConfig,
    selected_index: usize,
    edit_mode: Option<FolderEditMode>,
    input_buffer: String,
    skin_options: Vec<SkinOption>,
    selected_skin_index: usize,
    scan_requested: bool,
    message: Option<String>,
}

impl LibraryConfigScreen {
    /// Create a new library config screen.
    /// ライブラリ設定画面を作成する。
    pub fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let (skin_options, selected_skin_index) = Self::collect_skin_options(&config);
        Self {
            config,
            selected_index: 0,
            edit_mode: None,
            input_buffer: String::new(),
            skin_options,
            selected_skin_index,
            scan_requested: false,
            message: None,
        }
    }

    /// Update the screen. Returns a result when leaving the screen.
    /// 画面を更新し、離脱時に結果を返す。
    pub fn update(
        &mut self,
        hotkeys: &HotkeyConfig,
        input_manager: &InputManager,
    ) -> LibraryConfigResult {
        if self.edit_mode.is_some() {
            return self.update_edit_mode(hotkeys, input_manager);
        }

        let confirm = hotkeys.pressed_config(ConfigHotkey::Confirm)
            || input_manager.is_start_pressed()
            || is_key_pressed(KeyCode::Enter);
        let cancel = hotkeys.pressed_config(ConfigHotkey::Cancel)
            || input_manager.is_select_pressed()
            || is_key_pressed(KeyCode::Escape);
        let move_up = hotkeys.pressed_config(ConfigHotkey::Up) || is_key_pressed(KeyCode::Up);
        let move_down = hotkeys.pressed_config(ConfigHotkey::Down) || is_key_pressed(KeyCode::Down);

        let item_count = self.item_count();
        if move_up {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
        if move_down {
            self.selected_index = (self.selected_index + 1).min(item_count.saturating_sub(1));
        }

        if cancel {
            self.scan_requested = false;
            self.message = None;
            return LibraryConfigResult::Back { rescan: false };
        }

        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
            if let LibraryItem::Folder(index) = self.item_at(self.selected_index) {
                if index < self.config.song_folders.len() {
                    self.config.song_folders.remove(index);
                    if self.selected_index >= self.item_count() {
                        self.selected_index = self.item_count().saturating_sub(1);
                    }
                }
            }
        }

        if !confirm {
            return LibraryConfigResult::None;
        }

        match self.item_at(self.selected_index) {
            LibraryItem::Folder(index) => self.start_folder_edit(FolderEditMode::Edit(index)),
            LibraryItem::AddFolder => self.start_folder_edit(FolderEditMode::Add),
            LibraryItem::AutoScan => {
                self.config.auto_scan_on_startup = !self.config.auto_scan_on_startup;
            }
            LibraryItem::Skin => {
                if !self.skin_options.is_empty() {
                    self.selected_skin_index =
                        (self.selected_skin_index + 1) % self.skin_options.len();
                    self.config.play_skin_path =
                        self.skin_options[self.selected_skin_index].path.clone();
                }
            }
            LibraryItem::ScanNow => {
                self.scan_requested = true;
                self.message = Some("Scan requested. / スキャンを予約しました。".to_string());
            }
            LibraryItem::SaveAndBack => {
                if let Err(e) = self.config.save() {
                    self.message = Some(format!(
                        "Failed to save config: {} / 設定の保存に失敗: {}",
                        e, e
                    ));
                    return LibraryConfigResult::None;
                }
                let rescan = self.scan_requested;
                self.scan_requested = false;
                return LibraryConfigResult::Back { rescan };
            }
        }

        LibraryConfigResult::None
    }

    /// Draw the screen.
    /// 画面を描画する。
    pub fn draw(&self) {
        let x = 100.0;
        let mut y = 100.0;

        draw_text(
            "=== LIBRARY & SKIN / ライブラリ・スキン ===",
            x,
            y,
            42.0,
            WHITE,
        );
        y += 60.0;

        draw_text("SONG FOLDERS / 曲フォルダ", x, y, 24.0, YELLOW);
        y += 35.0;

        for (index, folder) in self.config.song_folders.iter().enumerate() {
            let item_index = index;
            let is_selected = self.selected_index == item_index;
            let color = if is_selected { YELLOW } else { WHITE };
            let prefix = if is_selected { "> " } else { "  " };
            let label = Self::truncate_text(folder, 60);
            draw_text(&format!("{}{}", prefix, label), x, y, 20.0, color);
            y += 28.0;
        }

        let add_index = self.config.song_folders.len();
        let is_add_selected = self.selected_index == add_index;
        let add_color = if is_add_selected { YELLOW } else { WHITE };
        let add_prefix = if is_add_selected { "> " } else { "  " };
        draw_text(
            &format!("{}Add Folder / フォルダ追加", add_prefix),
            x,
            y,
            20.0,
            add_color,
        );
        y += 36.0;

        let auto_index = add_index + 1;
        let is_auto_selected = self.selected_index == auto_index;
        let auto_color = if is_auto_selected { YELLOW } else { WHITE };
        let auto_prefix = if is_auto_selected { "> " } else { "  " };
        let auto_value = if self.config.auto_scan_on_startup {
            "ON / 有効"
        } else {
            "OFF / 無効"
        };
        draw_text(
            &format!(
                "{}Auto Scan on Start / 起動時スキャン: {}",
                auto_prefix, auto_value
            ),
            x,
            y,
            20.0,
            auto_color,
        );
        y += 28.0;

        let skin_index = auto_index + 1;
        let is_skin_selected = self.selected_index == skin_index;
        let skin_color = if is_skin_selected { YELLOW } else { WHITE };
        let skin_prefix = if is_skin_selected { "> " } else { "  " };
        let skin_label = self
            .skin_options
            .get(self.selected_skin_index)
            .map(|opt| opt.label.as_str())
            .unwrap_or("None / なし");
        let skin_label = Self::truncate_text(skin_label, 60);
        draw_text(
            &format!("{}Play Skin / プレイスキン: {}", skin_prefix, skin_label),
            x,
            y,
            20.0,
            skin_color,
        );
        y += 28.0;

        let scan_index = skin_index + 1;
        let is_scan_selected = self.selected_index == scan_index;
        let scan_color = if is_scan_selected { YELLOW } else { WHITE };
        let scan_prefix = if is_scan_selected { "> " } else { "  " };
        let scan_status = if self.scan_requested {
            " (Requested / 予約済み)"
        } else {
            ""
        };
        draw_text(
            &format!("{}Scan Now / 今すぐスキャン{}", scan_prefix, scan_status),
            x,
            y,
            20.0,
            scan_color,
        );
        y += 28.0;

        let save_index = scan_index + 1;
        let is_save_selected = self.selected_index == save_index;
        let save_color = if is_save_selected { YELLOW } else { WHITE };
        let save_prefix = if is_save_selected { "> " } else { "  " };
        draw_text(
            &format!("{}Save & Back / 保存して戻る", save_prefix),
            x,
            y,
            20.0,
            save_color,
        );
        y += 40.0;

        if let Some(message) = &self.message {
            draw_text(message, x, y, 18.0, GREEN);
            y += 26.0;
        }

        if let Some(edit_mode) = self.edit_mode {
            let label = match edit_mode {
                FolderEditMode::Add => "Add Folder / フォルダ追加",
                FolderEditMode::Edit(_) => "Edit Folder / フォルダ編集",
            };
            draw_text(label, x, y, 20.0, YELLOW);
            y += 26.0;
            draw_text(&self.input_buffer, x, y, 18.0, WHITE);
            y += 26.0;
            draw_text(
                "Enter: Save / 保存  Esc: Cancel / キャンセル",
                x,
                y,
                16.0,
                DARKGRAY,
            );
            return;
        }

        draw_text(
            "Up/Down: Navigate / 移動  Enter: Select / 決定  Esc: Cancel / キャンセル",
            x,
            y,
            16.0,
            DARKGRAY,
        );
        y += 20.0;
        draw_text("Delete: Remove Folder / フォルダ削除", x, y, 16.0, DARKGRAY);
    }

    fn update_edit_mode(
        &mut self,
        hotkeys: &HotkeyConfig,
        input_manager: &InputManager,
    ) -> LibraryConfigResult {
        let confirm = hotkeys.pressed_config(ConfigHotkey::Confirm)
            || input_manager.is_start_pressed()
            || is_key_pressed(KeyCode::Enter);
        let cancel = hotkeys.pressed_config(ConfigHotkey::Cancel)
            || input_manager.is_select_pressed()
            || is_key_pressed(KeyCode::Escape);

        while let Some(ch) = get_char_pressed() {
            if !ch.is_control() {
                self.input_buffer.push(ch);
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.input_buffer.pop();
        }

        if cancel {
            self.edit_mode = None;
            self.input_buffer.clear();
            return LibraryConfigResult::None;
        }

        if confirm {
            self.commit_folder_edit();
        }

        LibraryConfigResult::None
    }

    fn commit_folder_edit(&mut self) {
        let value = self.input_buffer.trim();
        if value.is_empty() {
            self.message = Some("Folder path is empty. / フォルダが空です。".to_string());
            return;
        }

        let value = value.to_string();
        match self.edit_mode {
            Some(FolderEditMode::Add) => {
                if !self.config.song_folders.iter().any(|p| p == &value) {
                    self.config.song_folders.push(value);
                }
            }
            Some(FolderEditMode::Edit(index)) => {
                if index < self.config.song_folders.len() {
                    self.config.song_folders[index] = value;
                }
            }
            None => {}
        }

        self.edit_mode = None;
        self.input_buffer.clear();
    }

    fn start_folder_edit(&mut self, mode: FolderEditMode) {
        self.input_buffer = match mode {
            FolderEditMode::Add => String::new(),
            FolderEditMode::Edit(index) => self
                .config
                .song_folders
                .get(index)
                .cloned()
                .unwrap_or_default(),
        };
        self.edit_mode = Some(mode);
    }

    fn item_count(&self) -> usize {
        self.config.song_folders.len() + 5
    }

    fn item_at(&self, index: usize) -> LibraryItem {
        let folder_count = self.config.song_folders.len();
        if index < folder_count {
            return LibraryItem::Folder(index);
        }
        let offset = index.saturating_sub(folder_count);
        match offset {
            0 => LibraryItem::AddFolder,
            1 => LibraryItem::AutoScan,
            2 => LibraryItem::Skin,
            3 => LibraryItem::ScanNow,
            _ => LibraryItem::SaveAndBack,
        }
    }

    fn collect_skin_options(config: &AppConfig) -> (Vec<SkinOption>, usize) {
        let mut options = Vec::new();
        options.push(SkinOption {
            label: "None / なし".to_string(),
            path: None,
        });

        let mut paths = Self::find_skin_paths();
        paths.sort();
        paths.dedup();

        for path in paths {
            options.push(SkinOption {
                label: path.clone(),
                path: Some(path),
            });
        }

        let mut selected_index = 0;
        if let Some(selected_path) = &config.play_skin_path {
            let mut index = options
                .iter()
                .position(|opt| opt.path.as_deref() == Some(selected_path.as_str()));
            let mut resolved_selected = None;

            if index.is_none() {
                resolved_selected = skin_path::resolve_skin_path(Path::new(selected_path));
                if let Some(resolved_selected) = resolved_selected.as_ref() {
                    for (i, option) in options.iter().enumerate() {
                        let Some(option_path) = option.path.as_deref() else {
                            continue;
                        };
                        if let Some(resolved_option) =
                            skin_path::resolve_skin_path(Path::new(option_path))
                        {
                            if &resolved_option == resolved_selected {
                                index = Some(i);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(found) = index {
                selected_index = found;
            } else if resolved_selected.is_some() {
                options.push(SkinOption {
                    label: format!(
                        "Custom: {} / カスタム: {}",
                        selected_path, selected_path
                    ),
                    path: Some(selected_path.clone()),
                });
                selected_index = options.len().saturating_sub(1);
            } else {
                options.push(SkinOption {
                    label: format!(
                        "Missing: {} / 見つかりません: {}",
                        selected_path, selected_path
                    ),
                    path: Some(selected_path.clone()),
                });
                selected_index = options.len().saturating_sub(1);
            }
        }

        (options, selected_index)
    }

    fn find_skin_paths() -> Vec<String> {
        let Some(root) = skin_path::find_skin_root() else {
            return Vec::new();
        };
        let root_label = root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("skins");

        let mut paths = Vec::new();
        for entry in WalkDir::new(&root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext == "luaskin" || ext == "lr2skin" || ext == "csv" {
                let display_path = match path.strip_prefix(&root) {
                    Ok(relative) => Path::new(root_label).join(relative),
                    Err(_) => path.to_path_buf(),
                };
                paths.push(display_path.to_string_lossy().to_string());
            }
        }

        paths
    }

    fn truncate_text(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            return text.to_string();
        }
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

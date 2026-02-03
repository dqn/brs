use anyhow::Result;
use macroquad::prelude::*;
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::audio::PreviewPlayer;
use crate::config::AppConfig;
use crate::database::{ClearType, Database, ScanProgress, SongData, SongScanTask};
use crate::input::{HotkeyConfig, InputManager, SelectHotkey};
use crate::model::load_chart;
use crate::model::note::Lane;
use crate::state::select::FavoriteStore;
use crate::state::select::bar::Bar;
use crate::state::select::bar_manager::BarManager;
use ::rand::seq::SliceRandom;

/// Request type for song scanning on select screen.
/// 選曲画面でのスキャン要求種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectScanRequest {
    None,
    Auto,
    Manual,
}

/// Phase of the select screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectPhase {
    /// Loading song list from database.
    Loading,
    /// Actively selecting a song.
    Selecting,
    /// Song has been decided, transitioning to play.
    Decided,
}

/// Transition from the select screen.
#[derive(Debug, Clone)]
pub enum SelectTransition {
    /// No transition, stay in select screen.
    None,
    /// Transition to decide screen with the selected song.
    Decide(Box<SongData>),
    /// Transition to config screen.
    Config,
    /// Exit the application.
    Exit,
}

/// Main select state for the song selection screen.
pub struct SelectState {
    bar_manager: BarManager,
    input_manager: InputManager,
    phase: SelectPhase,
    song_db: Database,
    score_db: Database,
    transition: SelectTransition,
    preview_player: PreviewPlayer,
    current_preview_sha256: Option<String>,
    hotkeys: HotkeyConfig,
    favorites: FavoriteStore,
    preview_enabled: bool,
    show_details: bool,
    scan_task: Option<SongScanTask>,
    scan_result: Option<ScanProgress>,
    scan_error: Option<String>,
}

impl SelectState {
    /// Number of visible bars in the song list.
    const VISIBLE_BARS: usize = 15;

    /// Create a new SelectState.
    pub fn new(
        input_manager: InputManager,
        song_db: Database,
        score_db: Database,
        scan_request: SelectScanRequest,
    ) -> Result<Self> {
        let hotkeys = HotkeyConfig::load().unwrap_or_default();
        let favorites = FavoriteStore::load().unwrap_or_default();
        let mut state = Self {
            bar_manager: BarManager::new(),
            input_manager,
            phase: SelectPhase::Loading,
            song_db,
            score_db,
            transition: SelectTransition::None,
            preview_player: PreviewPlayer::new()?,
            current_preview_sha256: None,
            hotkeys,
            favorites,
            preview_enabled: true,
            show_details: true,
            scan_task: None,
            scan_result: None,
            scan_error: None,
        };

        state.maybe_start_scan(scan_request);

        Ok(state)
    }

    /// Get the current phase.
    pub fn phase(&self) -> SelectPhase {
        self.phase
    }

    /// Get the transition request.
    pub fn transition(&self) -> &SelectTransition {
        &self.transition
    }

    /// Take the transition request, resetting it to None.
    pub fn take_transition(&mut self) -> SelectTransition {
        std::mem::replace(&mut self.transition, SelectTransition::None)
    }

    /// Get a reference to the input manager.
    pub fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }

    /// Take the input manager from this state.
    pub fn take_input_manager(&mut self) -> InputManager {
        // Create a dummy input manager to swap
        let key_config = self.input_manager.key_config().clone();
        let dummy = InputManager::new(key_config).unwrap();
        std::mem::replace(&mut self.input_manager, dummy)
    }

    /// Set the input manager.
    pub fn set_input_manager(&mut self, input_manager: InputManager) {
        self.input_manager = input_manager;
    }

    fn maybe_start_scan(&mut self, request: SelectScanRequest) {
        if request == SelectScanRequest::None {
            return;
        }

        let config = self.load_app_config();
        if request == SelectScanRequest::Auto && !config.auto_scan_on_startup {
            return;
        }

        self.start_scan_with_config(config);
    }

    fn start_scan_with_config(&mut self, config: AppConfig) {
        if self.scan_task.is_some() {
            return;
        }

        let folders = Self::build_scan_folders(&config.song_folders);
        if folders.is_empty() {
            self.scan_error =
                Some("No song folders configured. / 曲フォルダが設定されていません。".to_string());
            return;
        }

        self.scan_task = Some(SongScanTask::start(PathBuf::from("song.db"), folders));
        self.scan_result = None;
        self.scan_error = None;
        self.phase = SelectPhase::Loading;
    }

    fn load_app_config(&self) -> AppConfig {
        AppConfig::load().unwrap_or_else(|e| {
            warn!("Failed to load config / 設定の読み込みに失敗: {}", e);
            AppConfig::default()
        })
    }

    fn build_scan_folders(folders: &[String]) -> Vec<PathBuf> {
        folders
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(Self::expand_path)
            .collect()
    }

    fn expand_path(path: &str) -> PathBuf {
        if path == "~" {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home);
            }
        }
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(stripped);
            }
        }
        PathBuf::from(path)
    }

    fn is_scanning(&self) -> bool {
        self.scan_task
            .as_ref()
            .is_some_and(|task| !task.is_complete())
    }

    fn is_cancel_requested(&self) -> bool {
        self.hotkeys.pressed_select(SelectHotkey::Cancel) || is_key_pressed(KeyCode::Escape)
    }

    /// Update the select state. Call once per frame.
    pub fn update(&mut self) -> Result<()> {
        self.input_manager.update();

        match self.phase {
            SelectPhase::Loading => {
                if self.is_cancel_requested() {
                    self.transition = SelectTransition::Exit;
                    return Ok(());
                }

                if let Some(task) = &self.scan_task {
                    if task.is_complete() {
                        let progress = task.progress();
                        let result = task.take_result();
                        self.scan_task = None;
                        self.scan_result = Some(progress);
                        if let Some(result) = result {
                            if let Err(e) = result {
                                self.scan_error =
                                    Some(format!("Scan failed: {} / スキャン失敗: {}", e, e));
                            }
                        }
                        self.bar_manager.load_songs(&self.song_db, &self.score_db)?;
                        self.bar_manager
                            .set_favorites(self.favorites.items().clone());
                        if self.preview_enabled {
                            self.update_preview();
                        }
                        self.phase = SelectPhase::Selecting;
                    }
                    return Ok(());
                }

                self.bar_manager.load_songs(&self.song_db, &self.score_db)?;
                self.bar_manager
                    .set_favorites(self.favorites.items().clone());
                if self.preview_enabled {
                    self.update_preview();
                }
                self.phase = SelectPhase::Selecting;
            }
            SelectPhase::Selecting => {
                self.process_input();
            }
            SelectPhase::Decided => {
                // Wait for transition to be taken
            }
        }

        Ok(())
    }

    fn process_input(&mut self) {
        let prev_cursor = self.bar_manager.cursor();

        // Up: Key2 (S key / index 2)
        if self.hotkeys.pressed_select(SelectHotkey::MoveUp)
            || self.input_manager.just_pressed(Lane::Key2)
            || is_key_pressed(KeyCode::Up)
        {
            self.bar_manager.move_up();
        }

        // Down: Key4 (D key / index 4)
        if self.hotkeys.pressed_select(SelectHotkey::MoveDown)
            || self.input_manager.just_pressed(Lane::Key4)
            || is_key_pressed(KeyCode::Down)
        {
            self.bar_manager.move_down();
        }

        if self.hotkeys.pressed_select(SelectHotkey::PageUp) {
            for _ in 0..Self::VISIBLE_BARS {
                self.bar_manager.move_up();
            }
        }

        if self.hotkeys.pressed_select(SelectHotkey::PageDown) {
            for _ in 0..Self::VISIBLE_BARS {
                self.bar_manager.move_down();
            }
        }

        if self.hotkeys.pressed_select(SelectHotkey::SortNext) {
            self.bar_manager.next_sort();
        }

        if self.hotkeys.pressed_select(SelectHotkey::SortPrev) {
            self.bar_manager.prev_sort();
        }

        if self.hotkeys.pressed_select(SelectHotkey::FilterNext) {
            self.bar_manager.next_filter();
        }

        if self.hotkeys.pressed_select(SelectHotkey::FilterPrev) {
            self.bar_manager.prev_filter();
        }

        if self.hotkeys.pressed_select(SelectHotkey::TogglePreview) {
            self.preview_enabled = !self.preview_enabled;
            if !self.preview_enabled {
                self.preview_player.stop();
            } else {
                self.update_preview();
            }
        }

        if self.hotkeys.pressed_select(SelectHotkey::ToggleDetails) {
            self.show_details = !self.show_details;
        }

        if self.hotkeys.pressed_select(SelectHotkey::ToggleFavorite) {
            if let Some(Bar::Song(song_bar)) = self.bar_manager.current_bar() {
                self.favorites.toggle(&song_bar.song.sha256);
                self.bar_manager
                    .set_favorites(self.favorites.items().clone());
                if let Err(e) = self.favorites.save() {
                    warn!("Failed to save favorites / お気に入りの保存に失敗: {}", e);
                }
            }
        }

        if self.hotkeys.pressed_select(SelectHotkey::RandomSelect) {
            self.random_select();
        }

        if self.hotkeys.pressed_select(SelectHotkey::ReloadDatabase) {
            let config = self.load_app_config();
            self.start_scan_with_config(config);
        }

        if self.hotkeys.pressed_select(SelectHotkey::OpenConfig) {
            self.transition = SelectTransition::Config;
        }

        if self.bar_manager.cursor() != prev_cursor && self.preview_enabled {
            self.update_preview();
        }

        // Enter: Start or Enter key
        if self.hotkeys.pressed_select(SelectHotkey::Decide)
            || self.input_manager.is_start_pressed()
            || is_key_pressed(KeyCode::Enter)
        {
            if let Some(bar) = self.bar_manager.current_bar() {
                if let Some(song_bar) = bar.as_song() {
                    self.preview_player.stop();
                    self.transition = SelectTransition::Decide(Box::new(song_bar.song.clone()));
                    self.phase = SelectPhase::Decided;
                }
            }
        }

        // Escape: Exit
        if self.hotkeys.pressed_select(SelectHotkey::Cancel) || is_key_pressed(KeyCode::Escape) {
            self.transition = SelectTransition::Exit;
        }
    }

    fn update_preview(&mut self) {
        let Some(Bar::Song(song_bar)) = self.bar_manager.current_bar() else {
            self.preview_player.stop();
            self.current_preview_sha256 = None;
            return;
        };

        if self
            .current_preview_sha256
            .as_deref()
            .is_some_and(|sha| sha == song_bar.song.sha256)
        {
            return;
        }

        self.current_preview_sha256 = Some(song_bar.song.sha256.clone());

        let loaded = match load_chart(&song_bar.song.path) {
            Ok(loaded) => loaded,
            Err(e) => {
                warn!("Failed to load chart for preview: {}", e);
                self.preview_player.stop();
                return;
            }
        };

        let preview = loaded
            .bms
            .music_info
            .preview_music
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        if let Some(preview) = preview {
            let base_dir = song_bar.song.path.parent().unwrap_or(Path::new("."));
            let preview_path = base_dir.join(preview);
            if let Err(e) = self.preview_player.play(&preview_path) {
                warn!("Failed to play preview: {}", e);
            }
        } else {
            self.preview_player.stop();
        }
    }

    fn random_select(&mut self) {
        if self.bar_manager.is_empty() {
            return;
        }

        let mut indices: Vec<usize> = self
            .bar_manager
            .bars()
            .iter()
            .enumerate()
            .filter(|(_, bar)| matches!(bar, Bar::Song(_)))
            .map(|(index, _)| index)
            .collect();

        if indices.is_empty() {
            indices = (0..self.bar_manager.len()).collect();
        }

        if indices.len() > 1 {
            let current = self.bar_manager.cursor();
            indices.retain(|index| *index != current);
        }

        let mut rng = ::rand::thread_rng();
        if let Some(index) = indices.choose(&mut rng) {
            self.bar_manager.set_cursor(*index);
            if self.preview_enabled {
                self.update_preview();
            }
        }
    }

    /// Draw the select screen.
    pub fn draw(&self) {
        self.draw_header();
        self.draw_song_list();
        self.draw_song_info();
        self.draw_footer();
    }

    fn draw_header(&self) {
        // Title
        draw_text("MUSIC SELECT / 選曲", 50.0, 50.0, 40.0, WHITE);

        // Position indicator
        if !self.bar_manager.is_empty() {
            let position_text = format!(
                "{} / {}",
                self.bar_manager.cursor() + 1,
                self.bar_manager.len()
            );
            draw_text(&position_text, 420.0, 50.0, 24.0, YELLOW);
        }

        let sort = self.bar_manager.sort_mode();
        let filter = self.bar_manager.filter_mode();
        let sort_text = format!(
            "SORT: {} ({})  FILTER: {} ({})",
            sort.label(),
            sort.label_ja(),
            filter.label(),
            filter.label_ja()
        );
        draw_text(&sort_text, 50.0, 80.0, 18.0, GRAY);

        let preview_text = if self.preview_enabled {
            "Preview: ON / プレビュー: ON"
        } else {
            "Preview: OFF / プレビュー: OFF"
        };
        let details_text = if self.show_details {
            "Details: ON / 詳細: ON"
        } else {
            "Details: OFF / 詳細: OFF"
        };
        let toggle_x = 700.0;
        draw_text(preview_text, toggle_x, 50.0, 18.0, GRAY);
        draw_text(details_text, toggle_x, 70.0, 18.0, GRAY);

        if let Some(progress) = &self.scan_result {
            let summary = format!(
                "Last scan: +{} ~{} -{} err {} / 最終スキャン: 追加{} 更新{} 削除{} エラー{}",
                progress.added,
                progress.updated,
                progress.deleted,
                progress.errors,
                progress.added,
                progress.updated,
                progress.deleted,
                progress.errors
            );
            draw_text(&summary, 50.0, 100.0, 16.0, GRAY);
        }
    }

    fn draw_song_list(&self) {
        let start_x = 50.0;
        let start_y = if self.scan_result.is_some() { 120.0 } else { 100.0 };
        let row_height = 40.0;

        if self.is_scanning() {
            self.draw_scan_progress(start_x, start_y);
            return;
        }

        if self.bar_manager.is_empty() {
            if let Some(error) = &self.scan_error {
                draw_text(error, start_x, start_y + 50.0, 20.0, RED);
            } else {
                draw_text(
                    "No songs found. / 曲が見つかりません。",
                    start_x,
                    start_y + 50.0,
                    24.0,
                    GRAY,
                );
                draw_text(
                    "Please add BMS files to the database. / BMSファイルを追加してください。",
                    start_x,
                    start_y + 80.0,
                    18.0,
                    GRAY,
                );
            }
            return;
        }

        let (start_index, visible_bars) = self.bar_manager.visible_bars(Self::VISIBLE_BARS);
        let cursor = self.bar_manager.cursor();

        for (i, bar) in visible_bars.iter().enumerate() {
            let global_index = start_index + i;
            let y = start_y + (i as f32) * row_height;
            let is_selected = global_index == cursor;

            // Background for selected bar
            if is_selected {
                draw_rectangle(start_x - 10.0, y - 25.0, 600.0, row_height - 5.0, DARKBLUE);
            }

            // Clear lamp
            let lamp_color = self.get_clear_lamp_color(bar);
            draw_rectangle(start_x - 5.0, y - 20.0, 5.0, 25.0, lamp_color);

            // Favorite marker
            if let Bar::Song(song_bar) = bar {
                if self.bar_manager.is_favorite(&song_bar.song.sha256) {
                    draw_text("★", start_x - 25.0, y, 20.0, YELLOW);
                }
            }

            // Title
            let title_color = if is_selected { WHITE } else { LIGHTGRAY };
            let title = bar.title();
            let display_title = if title.len() > 40 {
                format!("{}...", &title[..37])
            } else {
                title.to_string()
            };
            draw_text(&display_title, start_x + 10.0, y, 20.0, title_color);

            // Level (for songs)
            if let Bar::Song(song_bar) = bar {
                let level_text = format!("Lv.{}", song_bar.song.level);
                let level_color = self.get_level_color(song_bar.song.level);
                draw_text(&level_text, start_x + 500.0, y, 18.0, level_color);
            }
        }
    }

    fn draw_scan_progress(&self, start_x: f32, start_y: f32) {
        let Some(task) = &self.scan_task else {
            return;
        };
        let progress = task.progress();

        let title = progress
            .message
            .as_deref()
            .unwrap_or("Scanning songs... / 曲をスキャン中...");
        draw_text(title, start_x, start_y + 30.0, 22.0, YELLOW);

        if let Some(folder) = progress.current_folder.as_deref() {
            let folder_text = format!("Folder: {} / フォルダ: {}", folder, folder);
            draw_text(&folder_text, start_x, start_y + 60.0, 18.0, LIGHTGRAY);
        }

        if let Some(file) = progress.current_file.as_deref() {
            let file_text = format!("File: {} / ファイル: {}", file, file);
            draw_text(&file_text, start_x, start_y + 85.0, 16.0, GRAY);
        }

        let count_text = format!(
            "Added: {} Updated: {} Deleted: {} Errors: {} / 追加{} 更新{} 削除{} エラー{}",
            progress.added,
            progress.updated,
            progress.deleted,
            progress.errors,
            progress.added,
            progress.updated,
            progress.deleted,
            progress.errors
        );
        draw_text(&count_text, start_x, start_y + 115.0, 16.0, GRAY);
    }

    fn draw_song_info(&self) {
        let x = 700.0;
        let y = 100.0;

        if let Some(Bar::Song(song_bar)) = self.bar_manager.current_bar() {
            let song = &song_bar.song;

            // Title
            draw_text("SONG INFO / 楽曲情報", x, y, 24.0, YELLOW);
            draw_text(&song.title, x, y + 40.0, 22.0, WHITE);

            if !song.subtitle.is_empty() {
                draw_text(&song.subtitle, x, y + 65.0, 16.0, GRAY);
            }

            if !self.show_details {
                draw_text(
                    "Details hidden. / 詳細は非表示です。",
                    x,
                    y + 100.0,
                    16.0,
                    GRAY,
                );
                return;
            }

            // Artist
            draw_text(
                &format!("Artist / アーティスト: {}", song.artist),
                x,
                y + 100.0,
                18.0,
                LIGHTGRAY,
            );

            if !song.subartist.is_empty() {
                draw_text(&song.subartist, x, y + 120.0, 14.0, GRAY);
            }

            // Genre
            draw_text(
                &format!("Genre / ジャンル: {}", song.genre),
                x,
                y + 150.0,
                16.0,
                GRAY,
            );

            // BPM
            let bpm_range = if song.min_bpm == song.max_bpm {
                format!("{}", song.max_bpm)
            } else {
                format!("{} - {}", song.min_bpm, song.max_bpm)
            };
            draw_text(
                &format!("BPM / BPM: {}", bpm_range),
                x,
                y + 180.0,
                18.0,
                LIGHTGRAY,
            );

            // Notes
            draw_text(
                &format!("Notes / ノーツ: {}", song.notes),
                x,
                y + 210.0,
                18.0,
                LIGHTGRAY,
            );

            // Score info
            if let Some(ref score) = song_bar.score {
                draw_text("BEST SCORE / ベストスコア", x, y + 260.0, 20.0, YELLOW);

                let (clear_en, clear_ja) = Self::clear_type_labels(score.clear);
                let clear_color =
                    self.get_clear_lamp_color(&Bar::Song(Box::new((**song_bar).clone())));
                draw_text(
                    &format!("Clear: {} / クリア: {}", clear_en, clear_ja),
                    x,
                    y + 290.0,
                    18.0,
                    clear_color,
                );

                draw_text(
                    &format!("EX-SCORE / EXスコア: {}", score.ex_score),
                    x,
                    y + 315.0,
                    18.0,
                    WHITE,
                );
                draw_text(
                    &format!("MAX COMBO / 最大コンボ: {}", score.max_combo),
                    x,
                    y + 340.0,
                    18.0,
                    WHITE,
                );
                draw_text(
                    &format!("MIN BP / 最小BP: {}", score.min_bp),
                    x,
                    y + 365.0,
                    18.0,
                    WHITE,
                );
                draw_text(
                    &format!("Play Count / プレイ回数: {}", score.play_count),
                    x,
                    y + 395.0,
                    16.0,
                    GRAY,
                );
            } else {
                draw_text("NO PLAY / 未プレイ", x, y + 260.0, 20.0, GRAY);
            }
        }
    }

    fn draw_footer(&self) {
        let y = screen_height() - 50.0;

        draw_text("Controls / 操作:", 50.0, y, 18.0, GRAY);
        draw_text("Up/Down or S/D: Move / 移動", 190.0, y, 16.0, GRAY);
        draw_text("Enter/Start: Select / 決定", 430.0, y, 16.0, GRAY);
        draw_text("Escape: Exit / 終了", 680.0, y, 16.0, GRAY);

        let y2 = y + 20.0;
        draw_text(
            "F2/F3: Sort / ソート  F4/F5: Filter / フィルタ  F: Favorite / お気に入り  F12: Scan / スキャン",
            50.0,
            y2,
            16.0,
            GRAY,
        );
        let y3 = y2 + 20.0;
        draw_text(
            "P: Preview / プレビュー  Tab: Details / 詳細  R: Random / ランダム  F1: Config / 設定",
            50.0,
            y3,
            16.0,
            GRAY,
        );
    }

    fn get_clear_lamp_color(&self, bar: &Bar) -> Color {
        if let Bar::Song(song_bar) = bar {
            if let Some(ref score) = song_bar.score {
                return match score.clear {
                    ClearType::NoPlay => DARKGRAY,
                    ClearType::Failed => Color::new(0.5, 0.0, 0.0, 1.0),
                    ClearType::AssistEasy | ClearType::LightAssistEasy => {
                        Color::new(0.5, 0.0, 0.5, 1.0)
                    }
                    ClearType::Easy => Color::new(0.0, 0.5, 0.0, 1.0),
                    ClearType::Normal => Color::new(0.0, 0.0, 1.0, 1.0),
                    ClearType::Hard => Color::new(1.0, 1.0, 1.0, 1.0),
                    ClearType::ExHard => Color::new(1.0, 1.0, 0.0, 1.0),
                    ClearType::FullCombo => Color::new(0.0, 1.0, 1.0, 1.0),
                    ClearType::Perfect => Color::new(1.0, 0.5, 0.0, 1.0),
                    ClearType::Max => Color::new(1.0, 0.84, 0.0, 1.0),
                };
            }
        }
        DARKGRAY
    }

    fn get_level_color(&self, level: i32) -> Color {
        match level {
            0..=3 => Color::new(0.5, 1.0, 0.5, 1.0),
            4..=6 => Color::new(1.0, 1.0, 0.5, 1.0),
            7..=9 => Color::new(1.0, 0.7, 0.5, 1.0),
            10..=11 => Color::new(1.0, 0.5, 0.5, 1.0),
            _ => Color::new(1.0, 0.3, 0.3, 1.0),
        }
    }

    fn clear_type_labels(clear: ClearType) -> (&'static str, &'static str) {
        match clear {
            ClearType::NoPlay => ("NO PLAY", "未プレイ"),
            ClearType::Failed => ("FAILED", "失敗"),
            ClearType::AssistEasy => ("ASSIST EASY", "アシストイージー"),
            ClearType::LightAssistEasy => ("L-ASSIST EASY", "ライトアシストイージー"),
            ClearType::Easy => ("EASY", "イージー"),
            ClearType::Normal => ("NORMAL", "ノーマル"),
            ClearType::Hard => ("HARD", "ハード"),
            ClearType::ExHard => ("EX-HARD", "EXハード"),
            ClearType::FullCombo => ("FULL COMBO", "フルコンボ"),
            ClearType::Perfect => ("PERFECT", "パーフェクト"),
            ClearType::Max => ("MAX", "MAX"),
        }
    }
}

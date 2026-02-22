use std::fs;
use std::path::{Path, PathBuf};

use crate::main_controller::MainController;
use crate::main_state::{MainState, MainStateData, MainStateType};
use crate::player_config::PlayerConfig;
use crate::timer_manager::TimerManager;
use beatoraja_types::skin_type::SkinType;

/// Skin configuration screen.
/// Translated from Java: SkinConfiguration extends MainState
///
/// This class is heavily dependent on libGDX skin loading (JSONSkinLoader, LuaSkinLoader,
/// LR2SkinHeaderLoader) and MainState base class. Most functionality is stubbed pending
/// Phase 5+ skin system integration.
#[allow(dead_code)]
pub struct SkinConfiguration {
    state_data: MainStateData,
    skin_type: Option<SkinType>,
    selected_skin_index: i32,
    custom_option_offset: i32,
    custom_option_offset_max: i32,
    // Phase 5+ types stubbed
}

impl SkinConfiguration {
    pub fn new(_main: &MainController, _player: &PlayerConfig) -> Self {
        Self {
            state_data: MainStateData::new(TimerManager::new()),
            skin_type: None,
            selected_skin_index: -1,
            custom_option_offset: 0,
            custom_option_offset_max: 0,
        }
    }

    pub fn create_internal(&mut self) {
        // TODO: loadSkin(SkinType::SKIN_SELECT), loadAllSkins, changeSkinType
        // Requires Phase 5+ skin system
        log::warn!("not yet implemented: SkinConfiguration::create requires Phase 5+ skin system");
    }

    pub fn render_internal(&mut self) {
        // TODO: input handling and rendering
        // Requires Phase 5+ MainState, ControlKeys
        log::warn!("not yet implemented: SkinConfiguration::render requires Phase 5+ UI types");
    }

    pub fn input_internal(&mut self) {
        // TODO: scroll input handling
        // Requires Phase 5+ BMSPlayerInputProcessor
    }

    pub fn get_skin_type(&self) -> Option<SkinType> {
        self.skin_type
    }

    pub fn get_skin_select_position(&self) -> f32 {
        if self.custom_option_offset_max == 0 {
            0.0
        } else {
            self.custom_option_offset as f32 / self.custom_option_offset_max as f32
        }
    }

    pub fn set_skin_select_position(&mut self, value: f32) {
        if (0.0..1.0).contains(&value) {
            self.custom_option_offset = (self.custom_option_offset_max as f32 * value) as i32;
        }
    }

    pub fn get_category_name(&self, _index: usize) -> &str {
        // TODO: requires custom_options list
        ""
    }

    pub fn get_display_value(&self, _index: usize) -> &str {
        // TODO: requires custom_options list
        ""
    }

    pub fn execute_event(&mut self, _id: i32, _arg1: i32, _arg2: i32) {
        // TODO: skin property events
        // Requires Phase 5+ SkinProperty, SkinPropertyMapper
    }

    pub fn dispose_resources(&mut self) {
        // TODO: dispose resources
    }

    /// Scan skin files recursively.
    #[allow(dead_code)]
    fn scan_skins(path: &Path, paths: &mut Vec<PathBuf>) {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    Self::scan_skins(&entry.path(), paths);
                }
            }
        } else {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if name.ends_with(".lr2skin") || name.ends_with(".luaskin") || name.ends_with(".json") {
                paths.push(path.to_path_buf());
            }
        }
    }
}

impl MainState for SkinConfiguration {
    fn state_type(&self) -> Option<MainStateType> {
        Some(MainStateType::SkinConfig)
    }

    fn main_state_data(&self) -> &MainStateData {
        &self.state_data
    }

    fn main_state_data_mut(&mut self) -> &mut MainStateData {
        &mut self.state_data
    }

    fn create(&mut self) {
        log::warn!(
            "TODO: Phase 22 - SkinConfiguration::create (loadSkin, loadAllSkins, changeSkinType)"
        );
    }

    fn render(&mut self) {
        log::warn!("TODO: Phase 22 - SkinConfiguration::render (input handling and rendering)");
    }

    fn input(&mut self) {
        log::warn!("TODO: Phase 22 - SkinConfiguration::input (scroll input handling)");
    }

    fn dispose(&mut self) {
        self.dispose_resources();
        // Call default trait dispose for skin/stage cleanup
        let data = self.main_state_data_mut();
        data.skin = None;
        data.stage = None;
    }
}

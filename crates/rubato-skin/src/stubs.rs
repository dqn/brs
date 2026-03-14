// External dependency stubs for Phase 6 Skin System
// Rendering stubs (LibGDX types) are in rendering_stubs.rs, re-exported here for compatibility.

// Re-export all rendering stubs (LibGDX graphics types, file types, GL constants)
pub use crate::rendering_stubs::*;

// ============================================================
// Re-exports from extracted modules (backward compatibility)
// ============================================================

// MainState trait -- canonical definition in crate::main_state
pub use crate::main_state::MainState;

// Timer -- canonical definition in crate::skin_timer
pub use crate::skin_timer::Timer;

// Resolution -- canonical definition in crate::skin_resolution
pub use crate::skin_resolution::Resolution;

// SkinConfigOffset -- canonical definition in crate::skin_config_offset
pub use crate::skin_config_offset::SkinConfigOffset;

// SkinOffset -- re-exported from rubato-types (Phase 25d-2)
pub use rubato_types::skin_offset::SkinOffset;

// TimingDistribution -- re-exported from rubato-types (Phase 25d-2)
pub use rubato_types::timing_distribution::TimingDistribution;

// beatoraja.song types (re-exports)
pub use rubato_song::song_data::SongData;
pub use rubato_song::song_information::SongInformation;

// ============================================================
// Shadow types (stubs for cross-crate dependencies)
// These remain until callers are migrated to SkinRenderContext methods.
// ============================================================

/// Stub for beatoraja.MainController
pub struct MainController {
    pub debug: bool,
}

impl MainController {
    pub fn input_processor(&self) -> &InputProcessor {
        static INPUT: std::sync::OnceLock<InputProcessor> = std::sync::OnceLock::new();
        INPUT.get_or_init(|| InputProcessor)
    }

    pub fn config(&self) -> &rubato_core::config::Config {
        static CONFIG: std::sync::OnceLock<rubato_core::config::Config> =
            std::sync::OnceLock::new();
        CONFIG.get_or_init(rubato_core::config::Config::default)
    }
}

/// Stub for input processor
pub struct InputProcessor;

// SAFETY: InputProcessor is a stateless unit struct with no fields.
// It contains no non-Send/Sync types; the impls are needed because
// it is stored behind OnceLock which requires Send + Sync.
unsafe impl Send for InputProcessor {}
unsafe impl Sync for InputProcessor {}

impl InputProcessor {
    pub fn mouse_x(&self) -> f32 {
        0.0
    }
    pub fn mouse_y(&self) -> f32 {
        0.0
    }
}

// ============================================================
// beatoraja.play types (stubs)
// ============================================================

/// Stub for beatoraja.play.BMSPlayer
pub struct BMSPlayer {
    pub judge_manager: JudgeManager,
}

impl BMSPlayer {
    pub fn skin_type(&self) -> crate::skin_type::SkinType {
        crate::skin_type::SkinType::Play7Keys
    }

    pub fn past_notes(&self) -> i32 {
        0
    }

    pub fn judge_manager(&self) -> &JudgeManager {
        &self.judge_manager
    }
}

/// Stub for beatoraja.play.JudgeManager (minimal for visualizers)
pub struct JudgeManager {
    pub recent_judges: Vec<i64>,
    pub recent_judges_index: usize,
}

impl JudgeManager {
    pub fn recent_judges_index(&self) -> usize {
        self.recent_judges_index
    }

    pub fn recent_judges(&self) -> &[i64] {
        &self.recent_judges
    }
}

/// Stub for beatoraja.result.MusicResult
pub struct MusicResult {
    pub resource: MusicResultResource,
}

impl MusicResult {
    pub fn timing_distribution(&self) -> &TimingDistribution {
        static DEFAULT: std::sync::OnceLock<TimingDistribution> = std::sync::OnceLock::new();
        DEFAULT.get_or_init(|| TimingDistribution {
            distribution: vec![],
            array_center: 0,
            average: 0.0,
            std_dev: 0.0,
        })
    }
}

/// Stub for PlayerResource within MusicResult context
pub struct MusicResultResource;

impl MusicResultResource {
    pub fn bms_model(&self) -> &bms_model::bms_model::BMSModel {
        static MODEL: std::sync::OnceLock<bms_model::bms_model::BMSModel> =
            std::sync::OnceLock::new();
        MODEL.get_or_init(bms_model::bms_model::BMSModel::default)
    }

    pub fn original_mode(&self) -> bms_model::mode::Mode {
        bms_model::mode::Mode::BEAT_7K
    }

    pub fn player_config(&self) -> &rubato_core::player_config::PlayerConfig {
        static PC: std::sync::OnceLock<rubato_core::player_config::PlayerConfig> =
            std::sync::OnceLock::new();
        PC.get_or_init(rubato_core::player_config::PlayerConfig::default)
    }

    pub fn constraint(&self) -> Vec<rubato_core::course_data::CourseDataConstraint> {
        vec![]
    }
}

/// Stub for beatoraja.PlayerResource
pub struct PlayerResource;

impl PlayerResource {
    pub fn songdata(&self) -> Option<&SongData> {
        None
    }

    pub fn bms_model(&self) -> &bms_model::bms_model::BMSModel {
        static MODEL: std::sync::OnceLock<bms_model::bms_model::BMSModel> =
            std::sync::OnceLock::new();
        MODEL.get_or_init(bms_model::bms_model::BMSModel::default)
    }

    pub fn original_mode(&self) -> bms_model::mode::Mode {
        bms_model::mode::Mode::BEAT_7K
    }

    pub fn player_config(&self) -> &rubato_core::player_config::PlayerConfig {
        static PC: std::sync::OnceLock<rubato_core::player_config::PlayerConfig> =
            std::sync::OnceLock::new();
        PC.get_or_init(rubato_core::player_config::PlayerConfig::default)
    }

    pub fn config(&self) -> &rubato_core::config::Config {
        static CFG: std::sync::OnceLock<rubato_core::config::Config> = std::sync::OnceLock::new();
        CFG.get_or_init(rubato_core::config::Config::default)
    }

    pub fn constraint(&self) -> Vec<rubato_core::course_data::CourseDataConstraint> {
        vec![]
    }
}

/// Stub for beatoraja.play.PlaySkin
pub struct PlaySkinStub {
    pub pomyu: rubato_play::pomyu_chara_processor::PomyuCharaProcessor,
}

impl Default for PlaySkinStub {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaySkinStub {
    pub fn new() -> Self {
        Self {
            pomyu: rubato_play::pomyu_chara_processor::PomyuCharaProcessor::new(),
        }
    }

    pub fn add(&mut self, _obj: crate::skin_image::SkinImage) {
        // stub
    }
}

/// Stub for beatoraja.skin.SkinLoader (static methods)
pub struct SkinLoaderStub;

impl SkinLoaderStub {
    pub fn texture(path: &str, usecim: bool) -> Option<Texture> {
        crate::skin_loader::texture(path, usecim)
    }
}

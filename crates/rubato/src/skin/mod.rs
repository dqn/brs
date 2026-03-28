//! Skin format loaders (LR2 CSV, JSON, Lua), property system,
//! rendering object hierarchy, and shared type definitions
//! (absorbed from rubato-types).

// ================================================================
// Modules absorbed from rubato-types
// ================================================================

// Semantic newtypes
pub mod event_id;
pub mod timer_id;
pub mod value_id;

// Enums and foundational types
pub mod bms_player_mode;
pub mod clear_type;
pub mod main_state_type;
pub mod screen_type;
pub mod skin_type_def;
/// Backwards-compatible alias for skin_type_def
pub use skin_type_def as skin_type;
pub mod sound_type;

// Config types
pub mod audio_config;
pub mod config;
pub mod ir_config;
pub mod play_config;
pub mod play_mode_config;
pub mod player_config;
pub mod skin_config;

// Data models
pub mod course_data;
pub mod folder_data;
pub mod player_data;
pub mod player_information;
pub mod replay_data;
pub mod score_data;
pub mod score_data_property;
pub mod song_data;
pub mod song_information;

// Skin contract types
pub mod distribution_data;
pub mod offset_capabilities;
pub mod property_snapshot;
pub mod skin_action_queue;
pub mod skin_main_state;
pub mod skin_offset;
pub mod skin_render_context;
pub mod skin_widget_focus;
pub mod timer_access;

// Play-side shared types
pub mod bga_types;
pub mod draw_command;
pub mod practice_draw_command;
pub mod skin_judge;
pub mod skin_note;

// Gameplay types
pub mod bar_sorter;
pub mod bm_keys;
pub mod bms_player_input_device;
pub mod bms_player_rule;
pub mod gauge_property;
pub mod groove_gauge;
pub mod judge_algorithm;
pub mod key_input_log;
pub mod last_played_sort;
pub mod long_note_modifier;
pub mod mine_note_modifier;
pub mod pattern_modify_log;
pub mod scroll_speed_modifier;
pub mod target_list;
pub mod timing_distribution;

// State and lifecycle
pub mod app_event;
pub mod fps_counter;
pub mod input_processor_access;
pub mod ipfs_information;
pub mod ir_connection_registry;
pub mod monotonic_clock;
pub mod player_resource_access;
pub mod resolution;
pub mod song_selection_access;
pub mod state_event;
pub mod sync_utils;
pub mod target_property_access;
pub mod validatable;

// Test support (behind feature gate)
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

// Top-level re-exports (matching rubato-types public API)
pub use bar_sorter::{BarSorter, BarSorterEntry};
pub use bms_player_rule::BMSPlayerRule;
pub use groove_gauge::GrooveGauge;
pub use ir_connection_registry::IRConnectionManager;
pub use judge_algorithm::JudgeAlgorithm;
pub use key_input_log::KeyInputLog;
pub use pattern_modify_log::PatternModifyLog;
pub use skin_type_def::SkinType;
pub use song_data::SongData;

// ================================================================
// Original rubato-skin modules
// ================================================================

// Property submodule (interfaces + factories)
pub mod property;

// Rendering re-exports (wgpu-backed LibGDX equivalents from rubato-render)
pub mod render_reexports;
// Re-exports (convenience imports for commonly used types)
pub mod reexports;

// Shared utilities (available to both loader and render code)
pub mod util;

// Bitmap font cache (shared between loader and render)
pub mod bitmap_font_cache;

// Real implementations and standalone types
pub mod main_state;
pub mod skin_config_offset;
pub mod skin_resolution;
pub mod skin_timer;
pub mod snapshot_main_state;

// Skin property enums (standalone, no subdir)
pub mod skin_property;

// Organized submodules
pub mod core;
pub mod graphs;
pub mod loaders;
pub mod objects;
pub mod sources;
pub mod text;
pub mod types;

// SkinDrawable trait (moved from rubato-render)
pub mod skin_drawable;

// Skin loaders
pub mod json;
pub mod lr2;
pub mod lua;

// Test helpers
#[cfg(test)]
pub(crate) mod test_helpers;

// Backwards-compatible re-exports for moved modules

// core/
pub use core::custom_event;
pub use core::custom_timer;
pub use core::float_formatter;
pub use core::skin_float;
pub use core::skin_property_mapper;
pub use core::stretch_type;

// types/
pub use types::select_bar_data;
pub use types::skin;
pub use types::skin_bar_object;
pub use types::skin_header;
pub use types::skin_node;
pub use types::skin_object;

// text/
pub use text::skin_text;
pub use text::skin_text_bitmap;
pub use text::skin_text_font;
pub use text::skin_text_image;

// objects/
pub use objects::skin_bga_object;
pub use objects::skin_gauge;
pub use objects::skin_gauge_graph_object;
pub use objects::skin_hidden;
pub use objects::skin_image;
pub use objects::skin_judge_object;
pub use objects::skin_note_object;
pub use objects::skin_number;
pub use objects::skin_slider;

// graphs/
pub use graphs::skin_bpm_graph;
pub use graphs::skin_graph;
pub use graphs::skin_hit_error_visualizer;
pub use graphs::skin_note_distribution_graph;
pub use graphs::skin_timing_distribution_graph;
pub use graphs::skin_timing_visualizer;

// loaders/
pub use loaders::bitmap_font_batch_loader;
// bitmap_font_cache is now a top-level module; loaders::bitmap_font_cache re-exports it
pub use loaders::pomyu_chara_loader;
pub use loaders::skin_data_converter;
pub use loaders::skin_loader;

// sources/
pub use sources::skin_source;
pub use sources::skin_source_image;
pub use sources::skin_source_image_set;
pub use sources::skin_source_movie;
pub use sources::skin_source_reference;
pub use sources::skin_source_set;

/// Division that returns 0.0 when the divisor is 0.0.
/// Prevents NaN/Inf from malformed skin data (e.g. zero-width src resolution).
#[inline]
pub(crate) fn safe_div_f32(a: f32, b: f32) -> f32 {
    if b == 0.0 { 0.0 } else { a / b }
}

#[cfg(test)]
mod safe_div_tests {
    use super::*;

    #[test]
    fn safe_div_f32_normal() {
        assert_eq!(safe_div_f32(10.0, 2.0), 5.0);
    }

    #[test]
    fn safe_div_f32_zero_divisor() {
        assert_eq!(safe_div_f32(10.0, 0.0), 0.0);
        assert_eq!(safe_div_f32(0.0, 0.0), 0.0);
        assert_eq!(safe_div_f32(-5.0, 0.0), 0.0);
    }

    #[test]
    fn safe_div_f32_negative_zero_divisor() {
        assert_eq!(safe_div_f32(10.0, -0.0), 0.0);
    }
}

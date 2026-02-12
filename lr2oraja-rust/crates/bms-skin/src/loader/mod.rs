// Skin loader module.
//
// Provides loaders for different skin formats:
// - JSON (beatoraja native format)
// - LR2 CSV (LR2 skin format) — Phase 9-7
// - Lua (Lua scripted skins) — Phase 9-8
//
// State-specific LR2 loaders:
// - lr2_play_loader: Play screen commands (SRC_NOTE, SRC_JUDGE, etc.)
// - lr2_select_loader: Select screen commands (SRC_BAR_BODY, etc.)
// - lr2_result_loader: Result/CourseResult commands (SRC_GAUGECHART, etc.)

pub mod json_loader;
pub mod json_skin;
pub mod lr2_csv_loader;
pub mod lr2_header_loader;
pub mod lr2_play_loader;
pub mod lr2_result_loader;
pub mod lr2_select_loader;
pub mod lua_event_utility;
pub mod lua_loader;
pub mod lua_timer_utility;

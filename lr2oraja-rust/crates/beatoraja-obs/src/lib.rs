#![allow(clippy::needless_range_loop)]
#![allow(unused_parens)]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

// Re-exports (formerly in stubs.rs)
pub use beatoraja_types::imgui_notify::ImGuiNotify;

/// Constants from ObsConfigurationView (from beatoraja-launcher, not yet available)
pub const SCENE_NONE: &str = "(No Change)";
pub const ACTION_NONE: &str = "(Do Nothing)";

// OBS WebSocket modules
pub mod obs_listener;
pub mod obs_ws_client;

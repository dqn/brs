// Re-exports (formerly in stubs.rs)
pub use rubato_types::imgui_notify::ImGuiNotify;
pub use rubato_types::sync_utils::lock_or_recover;

/// Constants from ObsConfigurationView (from beatoraja-launcher, not yet available)
pub const SCENE_NONE: &str = "(No Change)";
pub const ACTION_NONE: &str = "(Do Nothing)";

// OBS WebSocket modules
pub mod obs_listener;
pub mod obs_ws_client;

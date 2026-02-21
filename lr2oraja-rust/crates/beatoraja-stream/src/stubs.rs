// External dependency stubs for beatoraja-stream
// These will be replaced with actual implementations when corresponding types are available.

/// Stub for beatoraja.MessageRenderer
pub struct MessageRenderer;

/// Stub for beatoraja.modmenu.ImGuiNotify
pub struct ImGuiNotify;

impl ImGuiNotify {
    pub fn info(msg: &str) {
        log::info!("{}", msg);
    }
    pub fn warning(msg: &str) {
        log::warn!("{}", msg);
    }
}

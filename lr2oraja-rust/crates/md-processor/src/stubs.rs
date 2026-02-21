// Stub types for Phase 4 dependencies

/// Stub for bms.player.beatoraja.Config
pub struct Config {
    pub override_download_url: Option<String>,
}

impl Config {
    pub fn get_override_download_url(&self) -> Option<&str> {
        self.override_download_url.as_deref()
    }
}

/// Stub for MainController reference
pub trait MainControllerRef: Send + Sync {
    fn update_song(&self, path: &str, force: bool);
}

/// Stub for ImGuiNotify (logging placeholder)
pub struct ImGuiNotify;

impl ImGuiNotify {
    pub fn info(msg: &str) {
        log::info!("[ImGuiNotify] {}", msg);
    }

    pub fn warning(msg: &str) {
        log::warn!("[ImGuiNotify] {}", msg);
    }

    pub fn error(msg: &str) {
        log::error!("[ImGuiNotify] {}", msg);
    }
}

// ModMenu overlay plugin (skeleton).
//
// Provides an in-game overlay menu for adjusting play settings on the fly.
// Uses egui for rendering. Full implementation deferred to a later phase.

/// Marker component for the mod menu overlay.
///
/// When added as a Bevy plugin, this will render an egui overlay
/// with play-mode adjustments (gauge, random, speed, etc.).
#[derive(Default)]
pub struct ModMenuPlugin {
    enabled: bool,
}

impl ModMenuPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_disabled() {
        let plugin = ModMenuPlugin::default();
        assert!(!plugin.is_enabled());
    }

    #[test]
    fn toggle_works() {
        let mut plugin = ModMenuPlugin::new();
        assert!(!plugin.is_enabled());
        plugin.toggle();
        assert!(plugin.is_enabled());
        plugin.toggle();
        assert!(!plugin.is_enabled());
    }

    #[test]
    fn set_enabled_works() {
        let mut plugin = ModMenuPlugin::new();
        plugin.set_enabled(true);
        assert!(plugin.is_enabled());
        plugin.set_enabled(false);
        assert!(!plugin.is_enabled());
    }
}

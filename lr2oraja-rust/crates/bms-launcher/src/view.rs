// Launcher view trait for egui panels.
//
// Each settings panel (player, skin, audio, input, IR, etc.) implements
// this trait. Full implementation deferred to a later phase.

use bms_config::Config;

/// A launcher settings panel.
///
/// Implementations provide a named panel that can render its UI and
/// apply changes to the configuration.
pub trait LauncherView {
    /// Display name for the panel tab.
    fn name(&self) -> &str;

    /// Whether this panel has unsaved changes.
    fn has_changes(&self) -> bool {
        false
    }

    /// Apply pending changes to the config.
    fn apply(&mut self, _config: &mut Config) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubView;

    impl LauncherView for StubView {
        fn name(&self) -> &str {
            "Stub"
        }
    }

    #[test]
    fn stub_view_defaults() {
        let view = StubView;
        assert_eq!(view.name(), "Stub");
        assert!(!view.has_changes());
    }
}

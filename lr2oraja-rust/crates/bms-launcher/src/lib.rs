// Settings GUI (egui)
//
// Provides the launcher UI for configuring player settings, skin selection,
// and other game options. Full implementation deferred to a later phase.

pub mod view;

use bms_config::Config;

/// Main launcher application (skeleton).
///
/// Will hold egui state and a collection of `LauncherView` panels.
pub struct LauncherApp {
    config: Config,
}

impl LauncherApp {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_app_round_trip_config() {
        let config = Config::default();
        let app = LauncherApp::new(config.clone());
        assert_eq!(app.config().window_width, config.window_width);
    }
}

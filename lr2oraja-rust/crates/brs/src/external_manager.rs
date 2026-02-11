use bms_config::Config;
use tracing::info;

/// Manages external integrations (Discord RPC, OBS, Streaming).
#[derive(Default)]
pub struct ExternalManager {
    discord_enabled: bool,
    obs_enabled: bool,
    stream_enabled: bool,
}

impl ExternalManager {
    /// Create from config. Does not connect immediately.
    pub fn new(config: &Config) -> Self {
        Self {
            discord_enabled: config.use_discord_rpc,
            obs_enabled: config.use_obs_ws,
            stream_enabled: false, // No streaming config field yet
        }
    }

    /// Called on state transitions to update external integrations.
    pub fn on_state_change(&self, state_name: &str, song_title: Option<&str>) {
        if self.discord_enabled {
            info!(
                "ExternalManager: Discord would update to state={}, song={:?}",
                state_name, song_title
            );
        }
        if self.obs_enabled {
            info!(
                "ExternalManager: OBS would switch scene for state={}",
                state_name
            );
        }
    }

    /// Called on shutdown to clean up connections.
    pub fn shutdown(&self) {
        if self.stream_enabled {
            info!("ExternalManager: Stream controller would stop");
        }
    }

    pub fn is_discord_enabled(&self) -> bool {
        self.discord_enabled
    }

    pub fn is_obs_enabled(&self) -> bool {
        self.obs_enabled
    }

    pub fn is_stream_enabled(&self) -> bool {
        self.stream_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_disabled() {
        let mgr = ExternalManager::default();
        assert!(!mgr.is_discord_enabled());
        assert!(!mgr.is_obs_enabled());
        assert!(!mgr.is_stream_enabled());
    }

    #[test]
    fn new_from_default_config_all_disabled() {
        let config = Config::default();
        let mgr = ExternalManager::new(&config);
        assert!(!mgr.is_discord_enabled());
        assert!(!mgr.is_obs_enabled());
        assert!(!mgr.is_stream_enabled());
    }

    #[test]
    fn on_state_change_no_panic_when_disabled() {
        let mgr = ExternalManager::default();
        mgr.on_state_change("Play", Some("test song"));
        mgr.on_state_change("Result", None);
    }

    #[test]
    fn shutdown_no_panic_when_disabled() {
        let mgr = ExternalManager::default();
        mgr.shutdown();
    }

    #[test]
    fn is_methods_reflect_state() {
        let mgr = ExternalManager {
            discord_enabled: true,
            obs_enabled: false,
            stream_enabled: true,
        };
        assert!(mgr.is_discord_enabled());
        assert!(!mgr.is_obs_enabled());
        assert!(mgr.is_stream_enabled());
    }
}

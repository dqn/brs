use std::collections::HashMap;

use anyhow::Result;
use tracing::{info, warn};

use super::client::{ObsCommand, ObsWsClient};

/// State-driven OBS control that maps game states to OBS scenes/actions.
pub struct ObsListener {
    client: ObsWsClient,
    scene_map: HashMap<String, String>,
    action_map: HashMap<String, String>,
}

impl ObsListener {
    /// Create a new listener with scene and action mappings from config.
    pub fn new(
        client: ObsWsClient,
        scene_map: HashMap<String, String>,
        action_map: HashMap<String, String>,
    ) -> Self {
        Self {
            client,
            scene_map,
            action_map,
        }
    }

    /// Handle a game state change, switching OBS scenes/actions as configured.
    pub fn on_state_changed(&self, state_name: &str) -> Result<()> {
        // Switch scene if mapped
        if let Some(scene) = self.scene_map.get(state_name)
            && !scene.is_empty()
        {
            info!(
                "OBS: switching scene for state '{}' -> '{}'",
                state_name, scene
            );
            self.client.set_scene(scene)?;
        }

        // Execute action if mapped
        if let Some(action) = self.action_map.get(state_name) {
            match action.as_str() {
                "start_record" => {
                    info!("OBS: starting recording for state '{}'", state_name);
                    self.client.start_record()?;
                }
                "stop_record" => {
                    info!("OBS: stopping recording for state '{}'", state_name);
                    self.client.stop_record()?;
                }
                _ => {
                    warn!(
                        "OBS: unknown action '{}' for state '{}'",
                        action, state_name
                    );
                    self.client.send(ObsCommand::SendRequest {
                        request_type: action.clone(),
                        request_data: None,
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Connect OBS WebSocket.
    pub fn connect(&self, host: &str, port: u16, password: &str) -> Result<()> {
        self.client.connect(host, port, password)
    }

    /// Disconnect OBS WebSocket.
    pub fn disconnect(&self) -> Result<()> {
        self.client.disconnect()
    }
}

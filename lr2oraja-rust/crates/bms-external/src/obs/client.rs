use std::time::Duration;

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use super::protocol;

/// Commands that can be sent to the OBS WebSocket client.
#[derive(Debug)]
pub enum ObsCommand {
    /// Connect to OBS WebSocket server.
    Connect {
        host: String,
        port: u16,
        password: String,
    },
    /// Disconnect from OBS WebSocket server.
    Disconnect,
    /// Set the current program scene.
    SetScene(String),
    /// Start recording.
    StartRecord,
    /// Stop recording.
    StopRecord,
    /// Send a custom request.
    SendRequest {
        request_type: String,
        request_data: Option<serde_json::Value>,
    },
}

/// OBS recording mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObsRecordingMode {
    /// Keep all recordings.
    #[default]
    KeepAll = 0,
    /// Start/stop on screenshot.
    OnScreenshot = 1,
    /// Start/stop on replay save.
    OnReplay = 2,
}

impl ObsRecordingMode {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::OnScreenshot,
            2 => Self::OnReplay,
            _ => Self::KeepAll,
        }
    }
}

/// OBS WebSocket client that runs a background task for connection management.
pub struct ObsWsClient {
    command_tx: mpsc::UnboundedSender<ObsCommand>,
}

impl Default for ObsWsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ObsWsClient {
    /// Create a new client and spawn the background connection task.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(connection_task(rx));
        Self { command_tx: tx }
    }

    /// Send a command to the background task.
    pub fn send(&self, cmd: ObsCommand) -> Result<()> {
        self.command_tx
            .send(cmd)
            .map_err(|_| anyhow!("OBS command channel closed"))
    }

    /// Connect to OBS WebSocket server.
    pub fn connect(&self, host: &str, port: u16, password: &str) -> Result<()> {
        self.send(ObsCommand::Connect {
            host: host.to_string(),
            port,
            password: password.to_string(),
        })
    }

    /// Disconnect from OBS.
    pub fn disconnect(&self) -> Result<()> {
        self.send(ObsCommand::Disconnect)
    }

    /// Set the current scene.
    pub fn set_scene(&self, scene: &str) -> Result<()> {
        self.send(ObsCommand::SetScene(scene.to_string()))
    }

    /// Start recording.
    pub fn start_record(&self) -> Result<()> {
        self.send(ObsCommand::StartRecord)
    }

    /// Stop recording.
    pub fn stop_record(&self) -> Result<()> {
        self.send(ObsCommand::StopRecord)
    }
}

/// Background task that manages the WebSocket connection.
async fn connection_task(mut rx: mpsc::UnboundedReceiver<ObsCommand>) {
    let mut request_counter: u64 = 0;

    while let Some(cmd) = rx.recv().await {
        match cmd {
            ObsCommand::Connect {
                host,
                port,
                password,
            } => {
                info!("OBS WebSocket: connecting to {}:{}", host, port);
                if let Err(e) = try_connect(&host, port, &password).await {
                    error!("OBS WebSocket connection failed: {}", e);
                }
            }
            ObsCommand::Disconnect => {
                info!("OBS WebSocket: disconnecting");
            }
            ObsCommand::SetScene(scene) => {
                request_counter += 1;
                let _req = protocol::create_request(
                    "SetCurrentProgramScene",
                    &format!("req-{}", request_counter),
                    Some(serde_json::json!({"sceneName": scene})),
                );
                info!("OBS: set scene to {}", scene);
            }
            ObsCommand::StartRecord => {
                request_counter += 1;
                let _req = protocol::create_request(
                    "StartRecord",
                    &format!("req-{}", request_counter),
                    None,
                );
                info!("OBS: start recording");
            }
            ObsCommand::StopRecord => {
                request_counter += 1;
                let _req = protocol::create_request(
                    "StopRecord",
                    &format!("req-{}", request_counter),
                    None,
                );
                info!("OBS: stop recording");
            }
            ObsCommand::SendRequest {
                request_type,
                request_data,
            } => {
                request_counter += 1;
                let _req = protocol::create_request(
                    &request_type,
                    &format!("req-{}", request_counter),
                    request_data,
                );
                info!("OBS: send request {}", request_type);
            }
        }
    }
}

/// Maximum reconnect backoff.
const MAX_BACKOFF: Duration = Duration::from_secs(15);
/// Initial reconnect delay.
const INITIAL_BACKOFF: Duration = Duration::from_secs(2);

/// Attempt to connect to OBS WebSocket with exponential backoff.
async fn try_connect(host: &str, port: u16, password: &str) -> Result<()> {
    let url = format!("ws://{}:{}", host, port);
    let mut backoff = INITIAL_BACKOFF;

    for attempt in 1..=5 {
        match try_connect_once(&url, password).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                warn!(
                    "OBS connection attempt {} failed: {}. Retrying in {:?}",
                    attempt, e, backoff
                );
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
        }
    }

    Err(anyhow!("failed to connect to OBS after 5 attempts"))
}

/// Single connection attempt.
async fn try_connect_once(_url: &str, _password: &str) -> Result<()> {
    // Actual WebSocket connection would be here.
    // For now this is a stub - real implementation would use tokio-tungstenite.
    Err(anyhow!("OBS WebSocket connection not yet implemented"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_mode_from_i32() {
        assert_eq!(ObsRecordingMode::from_i32(0), ObsRecordingMode::KeepAll);
        assert_eq!(
            ObsRecordingMode::from_i32(1),
            ObsRecordingMode::OnScreenshot
        );
        assert_eq!(ObsRecordingMode::from_i32(2), ObsRecordingMode::OnReplay);
        assert_eq!(ObsRecordingMode::from_i32(-1), ObsRecordingMode::KeepAll);
        assert_eq!(ObsRecordingMode::from_i32(99), ObsRecordingMode::KeepAll);
    }

    #[test]
    fn recording_mode_default() {
        assert_eq!(ObsRecordingMode::default(), ObsRecordingMode::KeepAll);
    }
}

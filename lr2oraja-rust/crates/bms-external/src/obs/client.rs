use std::time::Duration;

use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use super::protocol;

type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;
type WsStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

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
    let mut sink: Option<WsSink> = None;
    let mut stream: Option<WsStream> = None;

    loop {
        // Build a future that reads from the stream only when connected.
        let stream_next = async {
            match stream.as_mut() {
                Some(s) => s.next().await,
                None => std::future::pending().await,
            }
        };

        tokio::select! {
            cmd = rx.recv() => {
                let Some(cmd) = cmd else { break };
                match cmd {
                    ObsCommand::Connect { host, port, password } => {
                        info!("OBS WebSocket: connecting to {}:{}", host, port);
                        // Drop existing connection before reconnecting.
                        sink = None;
                        stream = None;
                        match try_connect(&host, port, &password).await {
                            Ok((new_sink, new_stream)) => {
                                info!("OBS WebSocket: connected successfully");
                                sink = Some(new_sink);
                                stream = Some(new_stream);
                            }
                            Err(e) => {
                                error!("OBS WebSocket connection failed: {}", e);
                            }
                        }
                    }
                    ObsCommand::Disconnect => {
                        if let Some(ref mut s) = sink {
                            info!("OBS WebSocket: disconnecting");
                            let _ = s.close().await;
                        } else {
                            info!("OBS WebSocket: already disconnected");
                        }
                        sink = None;
                        stream = None;
                    }
                    cmd => {
                        if let Some(ref mut s) = sink {
                            if !send_command(cmd, s, &mut request_counter).await {
                                sink = None;
                                stream = None;
                            }
                        } else {
                            warn!("OBS WebSocket: not connected, ignoring command");
                        }
                    }
                }
            }
            msg = stream_next => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_server_message(&text);
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("OBS WebSocket: server closed connection");
                        sink = None;
                        stream = None;
                    }
                    Some(Err(e)) => {
                        warn!("OBS WebSocket read error: {}", e);
                        sink = None;
                        stream = None;
                    }
                    None => {
                        info!("OBS WebSocket: stream ended");
                        sink = None;
                        stream = None;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Send a request command via the WebSocket sink. Returns `true` if successful.
async fn send_command(cmd: ObsCommand, sink: &mut WsSink, request_counter: &mut u64) -> bool {
    let (request_type, request_data, label) = match cmd {
        ObsCommand::SetScene(scene) => (
            "SetCurrentProgramScene".to_string(),
            Some(serde_json::json!({"sceneName": scene})),
            format!("set scene to {}", scene),
        ),
        ObsCommand::StartRecord => (
            "StartRecord".to_string(),
            None,
            "start recording".to_string(),
        ),
        ObsCommand::StopRecord => ("StopRecord".to_string(), None, "stop recording".to_string()),
        ObsCommand::SendRequest {
            request_type,
            request_data,
        } => {
            let label = format!("send request {}", request_type);
            (request_type, request_data, label)
        }
        // Connect/Disconnect are handled by the caller.
        _ => return true,
    };

    *request_counter += 1;
    let req = protocol::create_request(
        &request_type,
        &format!("req-{}", request_counter),
        request_data,
    );
    if let Err(e) = sink.send(Message::Text(req)).await {
        warn!("OBS WebSocket: failed to {}: {}", label, e);
        return false;
    }
    info!("OBS: {}", label);
    true
}

/// Handle a message received from the OBS WebSocket server.
fn handle_server_message(text: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        warn!("OBS WebSocket: failed to parse server message");
        return;
    };

    let Some(op) = value.get("op").and_then(|v| v.as_u64()) else {
        warn!("OBS WebSocket: server message missing 'op' field");
        return;
    };

    match protocol::OpCode::from_u8(op as u8) {
        Some(protocol::OpCode::Event) => {
            if let Ok(event) = serde_json::from_value::<protocol::Event>(value) {
                info!("OBS event: {}", event.d.event_type);
            }
        }
        Some(protocol::OpCode::RequestResponse) => {
            if let Ok(resp) = serde_json::from_value::<protocol::RequestResponse>(value) {
                if resp.d.request_status.result {
                    info!(
                        "OBS request {} succeeded ({})",
                        resp.d.request_type, resp.d.request_id
                    );
                } else {
                    warn!(
                        "OBS request {} failed: code={}, comment={:?} ({})",
                        resp.d.request_type,
                        resp.d.request_status.code,
                        resp.d.request_status.comment,
                        resp.d.request_id
                    );
                }
            }
        }
        Some(other) => {
            warn!("OBS WebSocket: unexpected opcode {:?}", other);
        }
        None => {
            warn!("OBS WebSocket: unknown opcode {}", op);
        }
    }
}

/// Maximum reconnect backoff.
const MAX_BACKOFF: Duration = Duration::from_secs(15);
/// Initial reconnect delay.
const INITIAL_BACKOFF: Duration = Duration::from_secs(2);

/// Attempt to connect to OBS WebSocket with exponential backoff.
async fn try_connect(host: &str, port: u16, password: &str) -> Result<(WsSink, WsStream)> {
    let url = format!("ws://{}:{}", host, port);
    let mut backoff = INITIAL_BACKOFF;

    for attempt in 1..=5 {
        match try_connect_once(&url, password).await {
            Ok(conn) => return Ok(conn),
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

/// Single connection attempt with Hello/Identify handshake.
async fn try_connect_once(url: &str, password: &str) -> Result<(WsSink, WsStream)> {
    let (ws, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut sink, mut stream) = ws.split();

    // Read Hello message from server.
    let hello_msg = stream
        .next()
        .await
        .ok_or_else(|| anyhow!("connection closed before Hello"))?
        .map_err(|e| anyhow!("failed to read Hello: {}", e))?;

    let hello_text = hello_msg
        .into_text()
        .map_err(|e| anyhow!("Hello is not text: {}", e))?;

    let hello: protocol::Hello =
        serde_json::from_str(&hello_text).map_err(|e| anyhow!("failed to parse Hello: {}", e))?;

    info!(
        "OBS WebSocket: server version {}, rpc version {}",
        hello.d.obs_web_socket_version, hello.d.rpc_version
    );

    // Compute authentication if required.
    let auth = hello.d.authentication.map(|auth_challenge| {
        protocol::compute_auth(password, &auth_challenge.salt, &auth_challenge.challenge)
    });

    // Send Identify message.
    let identify = protocol::create_identify(hello.d.rpc_version, auth);
    sink.send(Message::Text(identify))
        .await
        .map_err(|e| anyhow!("failed to send Identify: {}", e))?;

    // Read Identified response.
    let identified_msg = stream
        .next()
        .await
        .ok_or_else(|| anyhow!("connection closed before Identified"))?
        .map_err(|e| anyhow!("failed to read Identified: {}", e))?;

    let identified_text = identified_msg
        .into_text()
        .map_err(|e| anyhow!("Identified is not text: {}", e))?;

    let identified: protocol::Identified = serde_json::from_str(&identified_text)
        .map_err(|e| anyhow!("failed to parse Identified: {}", e))?;

    info!(
        "OBS WebSocket: identified with rpc version {}",
        identified.d.negotiated_rpc_version
    );

    Ok((sink, stream))
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

    #[test]
    fn handle_server_message_event() {
        let json = r#"{"op":5,"d":{"eventType":"SceneChanged","eventData":{"sceneName":"Game"}}}"#;
        // Should not panic.
        handle_server_message(json);
    }

    #[test]
    fn handle_server_message_request_response_success() {
        let json = r#"{"op":7,"d":{"requestType":"GetSceneList","requestId":"req-1","requestStatus":{"result":true,"code":100}}}"#;
        handle_server_message(json);
    }

    #[test]
    fn handle_server_message_request_response_failure() {
        let json = r#"{"op":7,"d":{"requestType":"StartRecord","requestId":"req-2","requestStatus":{"result":false,"code":500,"comment":"Already recording"}}}"#;
        handle_server_message(json);
    }

    #[test]
    fn handle_server_message_unknown_opcode() {
        let json = r#"{"op":99,"d":{}}"#;
        handle_server_message(json);
    }

    #[test]
    fn handle_server_message_invalid_json() {
        handle_server_message("not json");
    }

    #[test]
    fn handle_server_message_missing_op() {
        let json = r#"{"d":{}}"#;
        handle_server_message(json);
    }

    #[test]
    fn handle_server_message_hello() {
        // Hello is unexpected in handle_server_message (only during handshake),
        // so it should log a warning about unexpected opcode.
        let json = r#"{"op":0,"d":{"obsWebSocketVersion":"5.0.0","rpcVersion":1}}"#;
        handle_server_message(json);
    }

    #[test]
    #[ignore] // Requires a running OBS WebSocket server
    fn try_connect_once_integration() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = try_connect_once("ws://localhost:4455", "").await;
            assert!(result.is_ok());
        });
    }
}

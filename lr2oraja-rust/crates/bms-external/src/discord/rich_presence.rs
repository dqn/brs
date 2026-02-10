use anyhow::{Result, anyhow};
use tracing::{info, warn};

use super::ipc::{IpcConnection, OpCode, decode_header, encode_packet};

/// Discord Rich Presence client.
pub struct RichPresenceClient<C: IpcConnection> {
    conn: C,
    app_id: String,
    connected: bool,
}

impl<C: IpcConnection> RichPresenceClient<C> {
    pub fn new(conn: C, app_id: &str) -> Self {
        Self {
            conn,
            app_id: app_id.to_string(),
            connected: false,
        }
    }

    /// Connect to Discord and send the handshake.
    pub async fn connect(&mut self) -> Result<()> {
        self.conn.connect().await?;

        let handshake = serde_json::json!({
            "v": 1,
            "client_id": self.app_id,
        });
        let payload = serde_json::to_string(&handshake)?;
        let packet = encode_packet(OpCode::Handshake, &payload);
        self.conn.write(&packet).await?;

        // Read handshake response
        let mut header = [0u8; 8];
        self.conn.read(&mut header).await?;
        let (opcode, length) = decode_header(&header)?;
        if opcode != OpCode::Frame {
            return Err(anyhow!("expected Frame response, got {:?}", opcode));
        }

        // Read and discard payload
        let mut response = vec![0u8; length as usize];
        self.conn.read(&mut response).await?;

        self.connected = true;
        info!("Discord RPC connected");
        Ok(())
    }

    /// Set the current activity (Rich Presence).
    pub async fn set_activity(
        &mut self,
        details: &str,
        state: &str,
        large_image: &str,
        large_text: &str,
    ) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("not connected to Discord"));
        }

        let nonce = uuid::Uuid::new_v4().to_string();
        let command = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": {
                    "details": details,
                    "state": state,
                    "assets": {
                        "large_image": large_image,
                        "large_text": large_text,
                    },
                },
            },
            "nonce": nonce,
        });

        let payload = serde_json::to_string(&command)?;
        let packet = encode_packet(OpCode::Frame, &payload);
        self.conn.write(&packet).await?;
        Ok(())
    }

    /// Close the RPC connection.
    pub async fn close(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        let packet = encode_packet(OpCode::Close, "{}");
        if let Err(e) = self.conn.write(&packet).await {
            warn!("error sending close packet: {}", e);
        }
        self.conn.close().await?;
        self.connected = false;
        info!("Discord RPC disconnected");
        Ok(())
    }

    /// Whether the client is currently connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use anyhow::Result;

    use super::*;
    use crate::discord::ipc::{IpcConnection, OpCode, encode_packet};

    /// Mock IPC connection for testing.
    struct MockIpc {
        written: Vec<Vec<u8>>,
        read_queue: VecDeque<Vec<u8>>,
        connected: bool,
    }

    impl MockIpc {
        fn new() -> Self {
            Self {
                written: Vec::new(),
                read_queue: VecDeque::new(),
                connected: false,
            }
        }

        /// Queue a full packet (header + payload) for reading.
        fn queue_response(&mut self, opcode: OpCode, payload: &str) {
            let packet = encode_packet(opcode, payload);
            // Split into header and payload for sequential reads
            self.read_queue.push_back(packet[..8].to_vec());
            self.read_queue.push_back(packet[8..].to_vec());
        }
    }

    impl IpcConnection for MockIpc {
        async fn connect(&mut self) -> Result<()> {
            self.connected = true;
            Ok(())
        }

        async fn write(&mut self, data: &[u8]) -> Result<()> {
            self.written.push(data.to_vec());
            Ok(())
        }

        async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if let Some(data) = self.read_queue.pop_front() {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                Ok(0)
            }
        }

        async fn close(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn connect_sends_handshake() {
        let mut mock = MockIpc::new();
        mock.queue_response(OpCode::Frame, r#"{"cmd":"DISPATCH","evt":"READY"}"#);

        let mut client = RichPresenceClient::new(mock, "1234567890");
        client.connect().await.unwrap();
        assert!(client.is_connected());

        // Verify handshake was written
        let written = &client.conn.written;
        assert_eq!(written.len(), 1);
        // First 4 bytes = opcode 0 (Handshake)
        assert_eq!(&written[0][0..4], &0u32.to_le_bytes());
    }

    #[tokio::test]
    async fn set_activity_sends_frame() {
        let mut mock = MockIpc::new();
        mock.queue_response(OpCode::Frame, r#"{"cmd":"DISPATCH","evt":"READY"}"#);

        let mut client = RichPresenceClient::new(mock, "1234567890");
        client.connect().await.unwrap();
        client
            .set_activity("details", "state", "icon", "tooltip")
            .await
            .unwrap();

        // 2 writes: handshake + set_activity
        assert_eq!(client.conn.written.len(), 2);
        // Second write should be Frame opcode
        assert_eq!(&client.conn.written[1][0..4], &1u32.to_le_bytes());
    }

    #[tokio::test]
    async fn set_activity_fails_when_not_connected() {
        let mock = MockIpc::new();
        let mut client = RichPresenceClient::new(mock, "1234567890");
        assert!(client.set_activity("a", "b", "c", "d").await.is_err());
    }

    #[tokio::test]
    async fn close_disconnects() {
        let mut mock = MockIpc::new();
        mock.queue_response(OpCode::Frame, r#"{"cmd":"DISPATCH","evt":"READY"}"#);

        let mut client = RichPresenceClient::new(mock, "1234567890");
        client.connect().await.unwrap();
        assert!(client.is_connected());

        client.close().await.unwrap();
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn close_noop_when_not_connected() {
        let mock = MockIpc::new();
        let mut client = RichPresenceClient::new(mock, "1234567890");
        // Should not fail
        client.close().await.unwrap();
    }
}

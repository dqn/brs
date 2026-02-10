use anyhow::{Result, anyhow};

/// Discord IPC opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum OpCode {
    Handshake = 0,
    Frame = 1,
    Close = 2,
    Ping = 3,
    Pong = 4,
}

impl OpCode {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Handshake),
            1 => Some(Self::Frame),
            2 => Some(Self::Close),
            3 => Some(Self::Ping),
            4 => Some(Self::Pong),
            _ => None,
        }
    }
}

/// Encode a Discord IPC packet: [opcode: u32 LE][length: u32 LE][payload: UTF-8 JSON]
pub fn encode_packet(opcode: OpCode, payload: &str) -> Vec<u8> {
    let payload_bytes = payload.as_bytes();
    let mut buf = Vec::with_capacity(8 + payload_bytes.len());
    buf.extend_from_slice(&(opcode as u32).to_le_bytes());
    buf.extend_from_slice(&(payload_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(payload_bytes);
    buf
}

/// Decode a Discord IPC packet header from 8 bytes.
/// Returns (opcode, payload_length).
pub fn decode_header(header: &[u8; 8]) -> Result<(OpCode, u32)> {
    let opcode_val = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    let opcode =
        OpCode::from_u32(opcode_val).ok_or_else(|| anyhow!("invalid opcode: {}", opcode_val))?;
    Ok((opcode, length))
}

/// IPC connection trait for Discord communication.
#[allow(async_fn_in_trait)]
pub trait IpcConnection: Send {
    /// Connect to the Discord IPC socket.
    async fn connect(&mut self) -> Result<()>;

    /// Write raw bytes to the IPC socket.
    async fn write(&mut self, data: &[u8]) -> Result<()>;

    /// Read raw bytes from the IPC socket.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Close the IPC connection.
    async fn close(&mut self) -> Result<()>;
}

/// Get the list of possible IPC socket paths for the current platform.
#[cfg(unix)]
pub fn ipc_paths() -> Vec<std::path::PathBuf> {
    let base_dirs: Vec<std::path::PathBuf> = vec![
        std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(std::path::PathBuf::from),
        std::env::var("TMPDIR").ok().map(std::path::PathBuf::from),
        Some(std::path::PathBuf::from("/tmp")),
    ]
    .into_iter()
    .flatten()
    .collect();

    let mut paths = Vec::new();
    for base in &base_dirs {
        for i in 0..10 {
            paths.push(base.join(format!("discord-ipc-{}", i)));
        }
    }
    paths
}

/// Get the list of possible IPC pipe paths for Windows.
#[cfg(windows)]
pub fn ipc_paths() -> Vec<std::path::PathBuf> {
    (0..10)
        .map(|i| std::path::PathBuf::from(format!(r"\\.\pipe\discord-ipc-{}", i)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_round_trip() {
        let opcodes = [
            (0u32, OpCode::Handshake),
            (1, OpCode::Frame),
            (2, OpCode::Close),
            (3, OpCode::Ping),
            (4, OpCode::Pong),
        ];
        for (val, expected) in opcodes {
            assert_eq!(OpCode::from_u32(val), Some(expected));
            assert_eq!(expected as u32, val);
        }
        assert_eq!(OpCode::from_u32(5), None);
        assert_eq!(OpCode::from_u32(255), None);
    }

    #[test]
    fn encode_packet_format() {
        let payload = r#"{"v":1}"#;
        let packet = encode_packet(OpCode::Handshake, payload);
        assert_eq!(packet.len(), 8 + payload.len());

        // opcode = 0 (LE)
        assert_eq!(&packet[0..4], &[0, 0, 0, 0]);
        // length = 7 (LE)
        assert_eq!(&packet[4..8], &[7, 0, 0, 0]);
        // payload
        assert_eq!(&packet[8..], payload.as_bytes());
    }

    #[test]
    fn decode_header_valid() {
        let mut header = [0u8; 8];
        header[0..4].copy_from_slice(&1u32.to_le_bytes()); // Frame
        header[4..8].copy_from_slice(&42u32.to_le_bytes()); // length
        let (opcode, length) = decode_header(&header).unwrap();
        assert_eq!(opcode, OpCode::Frame);
        assert_eq!(length, 42);
    }

    #[test]
    fn decode_header_invalid_opcode() {
        let mut header = [0u8; 8];
        header[0..4].copy_from_slice(&99u32.to_le_bytes());
        header[4..8].copy_from_slice(&0u32.to_le_bytes());
        assert!(decode_header(&header).is_err());
    }

    #[test]
    fn encode_empty_payload() {
        let packet = encode_packet(OpCode::Close, "");
        assert_eq!(packet.len(), 8);
        assert_eq!(&packet[0..4], &2u32.to_le_bytes());
        assert_eq!(&packet[4..8], &0u32.to_le_bytes());
    }

    #[test]
    fn ipc_paths_not_empty() {
        let paths = ipc_paths();
        assert!(!paths.is_empty());
    }
}

use anyhow::{Result, bail};

use crate::connection::IPCConnection;

/// Translates: WindowsIPCConnection.java
///
/// ```java
/// class WindowsIPCConnection implements IPCConnection {
///     private static final String PIPE_PATH = "\\\\.\\pipe\\discord-ipc-0";
///     private WinNT.HANDLE pipeHandle;
/// ```
///
/// Stub implementation for non-Windows platforms.
/// Full Windows named pipe implementation is deferred.
#[allow(dead_code)]
const PIPE_PATH: &str = r"\\.\pipe\discord-ipc-0";

pub struct WindowsIPCConnection {
    // Windows HANDLE would go here
}

impl WindowsIPCConnection {
    pub fn new() -> Self {
        WindowsIPCConnection {}
    }
}

impl Default for WindowsIPCConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl IPCConnection for WindowsIPCConnection {
    /// Translates:
    /// ```java
    /// public void connect() throws IOException {
    ///     pipeHandle = Kernel32.INSTANCE.CreateFile(PIPE_PATH,
    ///             Kernel32.GENERIC_READ | Kernel32.GENERIC_WRITE,
    ///             0, null, Kernel32.OPEN_EXISTING, 0, null);
    ///
    ///     if (pipeHandle == null || WinNT.INVALID_HANDLE_VALUE.equals(pipeHandle)) {
    ///         throw new IOException("Failed to connect to Discord IPC pipe");
    ///     }
    /// }
    /// ```
    fn connect(&mut self) -> Result<()> {
        bail!("Windows IPC connection not implemented on this platform");
    }

    /// Translates:
    /// ```java
    /// public void write(ByteBuffer buffer) throws IOException {
    ///     byte[] data = new byte[buffer.remaining()];
    ///     buffer.get(data);
    ///     IntByReference bytesWritten = new IntByReference();
    ///
    ///     if (!Kernel32.INSTANCE.WriteFile(pipeHandle, data, data.length, bytesWritten, null)) {
    ///         throw new IOException("Failed to write to Discord IPC pipe");
    ///     }
    /// }
    /// ```
    fn write(&mut self, _buffer: &[u8]) -> Result<()> {
        bail!("Windows IPC connection not implemented on this platform");
    }

    /// Translates:
    /// ```java
    /// public ByteBuffer read(int size) throws IOException {
    ///     byte[] data = new byte[size];
    ///     IntByReference bytesRead = new IntByReference();
    ///
    ///     if (!Kernel32.INSTANCE.ReadFile(pipeHandle, data, data.length, bytesRead, null)) {
    ///         throw new IOException("Failed to read from Discord IPC pipe");
    ///     }
    ///
    ///     return ByteBuffer.wrap(data, 0, bytesRead.getValue());
    /// }
    /// ```
    fn read(&mut self, _size: usize) -> Result<Vec<u8>> {
        bail!("Windows IPC connection not implemented on this platform");
    }

    /// Translates:
    /// ```java
    /// public void close() {
    ///     if (pipeHandle != null && !WinNT.INVALID_HANDLE_VALUE.equals(pipeHandle)) {
    ///         Kernel32.INSTANCE.CloseHandle(pipeHandle);
    ///     }
    /// }
    /// ```
    fn close(&mut self) {
        // No-op on non-Windows platforms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let _conn = WindowsIPCConnection::new();
    }

    #[test]
    fn test_default() {
        let _conn = WindowsIPCConnection::default();
    }

    #[test]
    fn test_connect_fails_on_non_windows() {
        let mut conn = WindowsIPCConnection::new();
        let result = conn.connect();
        assert!(result.is_err());
    }

    #[test]
    fn test_close_no_panic() {
        let mut conn = WindowsIPCConnection::new();
        conn.close();
    }
}

// Stream controller
//
// On Windows, connects to \\.\pipe\beatoraja named pipe and dispatches
// incoming lines to registered StreamCommand handlers.
// On non-Windows platforms, provides a stub implementation.

use std::sync::Arc;

use tokio::sync::watch;
use tracing::{info, warn};

use crate::command::StreamCommand;

/// Manages the named pipe connection and dispatches commands.
pub struct StreamController {
    commands: Vec<Arc<dyn StreamCommand + Send + Sync>>,
    shutdown_tx: Option<watch::Sender<bool>>,
}

impl StreamController {
    pub fn new(commands: Vec<Arc<dyn StreamCommand + Send + Sync>>) -> Self {
        Self {
            commands,
            shutdown_tx: None,
        }
    }

    /// Start the stream controller.
    ///
    /// On Windows, spawns a tokio task that reads from the named pipe.
    /// On other platforms, logs a warning and returns immediately.
    #[cfg(target_os = "windows")]
    pub fn start(&mut self) {
        let (tx, rx) = watch::channel(false);
        self.shutdown_tx = Some(tx);

        let commands = self.commands.clone();
        tokio::spawn(async move {
            if let Err(e) = run_pipe_listener(commands, rx).await {
                warn!("Stream pipe listener error: {}", e);
            }
        });

        info!("Stream controller started (Windows named pipe)");
    }

    /// Start the stream controller (non-Windows stub).
    #[cfg(not(target_os = "windows"))]
    pub fn start(&mut self) {
        warn!(
            "Stream controller is only supported on Windows (named pipe \\\\.\\ pipe\\beatoraja)"
        );
    }

    /// Stop the stream controller by sending a shutdown signal.
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
            info!("Stream controller stop signal sent");
        }
    }

    /// Return a reference to the registered commands.
    pub fn commands(&self) -> &[Arc<dyn StreamCommand + Send + Sync>] {
        &self.commands
    }
}

/// Dispatch a received line to all registered commands.
#[cfg(any(target_os = "windows", test))]
fn dispatch_line(commands: &[Arc<dyn StreamCommand + Send + Sync>], line: &str) {
    for cmd in commands {
        let prefix = cmd.command_string();
        let full_prefix = format!("{} ", prefix);
        if let Some(args) = line.strip_prefix(&full_prefix) {
            match cmd.run(args) {
                Ok(Some(response)) => {
                    info!("Command '{}' response: {}", prefix, response);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Command '{}' error: {}", prefix, e);
                }
            }
        } else if line == prefix {
            // Command with no arguments
            match cmd.run("") {
                Ok(Some(response)) => {
                    info!("Command '{}' response: {}", prefix, response);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Command '{}' error: {}", prefix, e);
                }
            }
        }
    }
}

/// Windows named pipe listener implementation.
#[cfg(target_os = "windows")]
async fn run_pipe_listener(
    commands: Vec<Arc<dyn StreamCommand + Send + Sync>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    use tokio::io::AsyncBufReadExt;
    use tokio::net::windows::named_pipe::ClientOptions;

    let pipe_name = r"\\.\pipe\beatoraja";
    info!("Connecting to named pipe: {}", pipe_name);

    let pipe = ClientOptions::new().open(pipe_name)?;
    let reader = tokio::io::BufReader::new(pipe);
    let mut lines = reader.lines();

    loop {
        tokio::select! {
            result = lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        info!("Received: {}", line);
                        dispatch_line(&commands, &line);
                    }
                    Ok(None) => {
                        info!("Named pipe closed");
                        break;
                    }
                    Err(e) => {
                        warn!("Error reading from pipe: {}", e);
                        break;
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::StreamRequestCommand;

    #[test]
    fn test_new_controller() {
        let cmd = Arc::new(StreamRequestCommand::default());
        let controller = StreamController::new(vec![cmd.clone()]);
        assert_eq!(controller.commands().len(), 1);
    }

    #[test]
    fn test_dispatch_line_with_args() {
        let cmd = Arc::new(StreamRequestCommand::default());
        let commands: Vec<Arc<dyn StreamCommand + Send + Sync>> = vec![cmd.clone()];

        let hash = "a".repeat(64);
        let line = format!("!!req {}", hash);
        dispatch_line(&commands, &line);

        assert_eq!(cmd.pending_count(), 1);
        let requests = cmd.poll_requests();
        assert_eq!(requests[0], hash);
    }

    #[test]
    fn test_dispatch_line_no_match() {
        let cmd = Arc::new(StreamRequestCommand::default());
        let commands: Vec<Arc<dyn StreamCommand + Send + Sync>> = vec![cmd.clone()];

        dispatch_line(&commands, "!!unknown something");
        assert_eq!(cmd.pending_count(), 0);
    }

    #[test]
    fn test_dispatch_line_no_args() {
        let cmd = Arc::new(StreamRequestCommand::default());
        let commands: Vec<Arc<dyn StreamCommand + Send + Sync>> = vec![cmd.clone()];

        // Command with no args (empty string) - hash validation will reject it
        dispatch_line(&commands, "!!req");
        assert_eq!(cmd.pending_count(), 0);
    }

    #[test]
    fn test_dispatch_multiple_commands() {
        let cmd1 = Arc::new(StreamRequestCommand::new(10));
        let cmd2 = Arc::new(StreamRequestCommand::new(10));
        let commands: Vec<Arc<dyn StreamCommand + Send + Sync>> = vec![cmd1.clone(), cmd2.clone()];

        let hash = "b".repeat(64);
        let line = format!("!!req {}", hash);
        dispatch_line(&commands, &line);

        // Both commands have the same prefix, so both receive the message
        assert_eq!(cmd1.pending_count(), 1);
        assert_eq!(cmd2.pending_count(), 1);
    }

    #[test]
    fn test_stop_without_start() {
        let cmd = Arc::new(StreamRequestCommand::default());
        let mut controller = StreamController::new(vec![cmd]);
        // Should not panic
        controller.stop();
    }

    #[test]
    fn test_dispatch_invalid_hash() {
        let cmd = Arc::new(StreamRequestCommand::default());
        let commands: Vec<Arc<dyn StreamCommand + Send + Sync>> = vec![cmd.clone()];

        dispatch_line(&commands, "!!req tooshort");
        assert_eq!(cmd.pending_count(), 0);
    }

    #[test]
    fn test_dispatch_line_extra_spaces() {
        let cmd = Arc::new(StreamRequestCommand::default());
        let commands: Vec<Arc<dyn StreamCommand + Send + Sync>> = vec![cmd.clone()];

        // Extra leading spaces in args - the command prefix match uses strip_prefix
        // so "!!req  <hash>" would have args " <hash>" with leading space,
        // which trim() in run() handles
        let hash = "c".repeat(64);
        let line = format!("!!req  {}", hash);
        dispatch_line(&commands, &line);
        // The args will be " <hash>" which trim() handles
        assert_eq!(cmd.pending_count(), 1);
    }
}

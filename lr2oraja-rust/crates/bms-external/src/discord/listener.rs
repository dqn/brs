use tracing::{info, warn};

use super::ipc::IpcConnection;
use super::rich_presence::RichPresenceClient;

/// Default Discord application ID for brs.
pub const APP_ID: &str = "1054234988167561277";

/// Discord state listener that updates Rich Presence based on game state.
pub struct DiscordListener<C: IpcConnection> {
    client: RichPresenceClient<C>,
}

impl<C: IpcConnection> DiscordListener<C> {
    pub fn new(client: RichPresenceClient<C>) -> Self {
        Self { client }
    }

    /// Update Discord Rich Presence based on state change.
    pub async fn on_state_changed(
        &mut self,
        state_name: &str,
        song_title: Option<&str>,
        details: Option<&str>,
    ) {
        let display_details = details.unwrap_or(state_name);
        let display_state = song_title.unwrap_or("");

        if let Err(e) = self
            .client
            .set_activity(display_details, display_state, "icon", "brs")
            .await
        {
            warn!("failed to update Discord presence: {}", e);
        } else {
            info!(
                "Discord presence updated: {} - {:?}",
                state_name, song_title
            );
        }
    }

    /// Connect the underlying RPC client.
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        self.client.connect().await
    }

    /// Disconnect the underlying RPC client.
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.client.close().await
    }

    /// Whether the listener is connected.
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_id_is_correct() {
        assert_eq!(APP_ID, "1054234988167561277");
    }
}

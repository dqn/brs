use crate::discord_rpc::rich_presence::{RichPresence, RichPresenceData};

use crate::stubs::{MainStateListener, ScreenType};
use rubato_types::main_state_access::MainStateAccess;

static APPLICATION_ID: &str = "1054234988167561277";

/// Discord Rich Presence listener.
/// Translated from Java: DiscordListener implements MainStateListener
pub struct DiscordListener {
    rich_presence: Option<RichPresence>,
    /// Cached start timestamp (Unix seconds), captured once per activity change
    start_timestamp: i64,
    /// Last observed screen type, used to detect activity changes
    last_screen_type: Option<ScreenType>,
}

impl DiscordListener {
    pub fn new() -> Self {
        let rich_presence = match Self::try_connect() {
            Ok(rp) => {
                log::info!("Discord RPC Ready!");
                Some(rp)
            }
            Err(e) => {
                log::warn!("Failed to initialize Discord RPC: {}", e);
                None
            }
        };
        Self {
            rich_presence,
            start_timestamp: 0,
            last_screen_type: None,
        }
    }

    fn try_connect() -> anyhow::Result<RichPresence> {
        let mut rp = RichPresence::new(APPLICATION_ID.to_string());
        rp.connect()?;
        Ok(rp)
    }

    pub fn close(&mut self) {
        if let Some(ref mut rp) = self.rich_presence {
            rp.close();
        }
    }
}

impl Default for DiscordListener {
    fn default() -> Self {
        Self::new()
    }
}

impl MainStateListener for DiscordListener {
    fn update(&mut self, state: &dyn MainStateAccess, _status: i32) {
        let rp = match self.rich_presence.as_mut() {
            Some(rp) => rp,
            None => return,
        };

        let result: Result<(), anyhow::Error> = (|| {
            let screen_type = state.screen_type();

            // Capture start_timestamp once when the activity (screen) changes
            if self.last_screen_type != Some(screen_type) {
                self.last_screen_type = Some(screen_type);
                self.start_timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
            }

            let mut data = RichPresenceData::new()
                .set_start_timestamp(self.start_timestamp)
                .set_large_image("bms".to_string(), String::new());

            match screen_type {
                ScreenType::MusicSelector => {
                    data = data.set_state("In Music Select Menu".to_string());
                }
                ScreenType::MusicDecide => {
                    data = data.set_state("Decide Screen".to_string());
                }
                ScreenType::BMSPlayer => {
                    if let Some(resource) = state.resource()
                        && let Some(songdata) = resource.songdata()
                    {
                        let full_title = if songdata.metadata.subtitle.is_empty() {
                            songdata.metadata.title.clone()
                        } else {
                            format!("{} {}", songdata.metadata.title, songdata.metadata.subtitle)
                        };
                        data = data
                            .set_details(format!("{} / {}", full_title, songdata.metadata.artist));
                        data = data.set_state(format!("Playing: {}Keys", songdata.chart.mode));
                    }
                }
                ScreenType::MusicResult => {
                    data = data.set_state("Result Screen".to_string());
                }
                ScreenType::CourseResult => {
                    data = data.set_state("Course Result Screen".to_string());
                }
                _ => {}
            }

            rp.update(data)?;
            Ok(())
        })();

        if let Err(e) = result {
            log::warn!("Failed to update Discord Rich Presence: {}", e);
        }
    }
}

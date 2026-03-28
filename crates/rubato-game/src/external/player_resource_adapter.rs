// PlayerResource wrapper delegating to concrete CorePlayerResource.

use crate::core::config::Config;
use crate::core::player_resource::PlayerResource as CorePlayerResource;
use crate::core::replay_data::ReplayData;
use crate::song::song_data::SongData;
use bms::model::mode::Mode;

/// Wrapper for bms.player.beatoraja.PlayerResource.
/// Delegates to concrete `CorePlayerResource` for trait methods.
/// `original_mode()` is crate-local (not on trait, since Mode lives in bms-model).
pub struct PlayerResource {
    pub(crate) inner: CorePlayerResource,
    original_mode: Mode,
}

impl PlayerResource {
    pub fn new(inner: CorePlayerResource, original_mode: Mode) -> Self {
        Self {
            inner,
            original_mode,
        }
    }

    pub fn config(&self) -> &Config {
        self.inner.config()
    }

    pub fn songdata(&self) -> Option<&SongData> {
        self.inner.songdata()
    }

    pub fn replay_data(&self) -> Option<&ReplayData> {
        self.inner.replay_data()
    }

    pub fn reverse_lookup_levels(&self) -> Vec<String> {
        self.inner.reverse_lookup_levels()
    }

    pub fn original_mode(&self) -> &Mode {
        &self.original_mode
    }
}

impl Default for PlayerResource {
    fn default() -> Self {
        Self {
            inner: CorePlayerResource::new(
                rubato_skin::config::Config::default(),
                rubato_skin::player_config::PlayerConfig::default(),
            ),
            original_mode: Mode::BEAT_7K,
        }
    }
}

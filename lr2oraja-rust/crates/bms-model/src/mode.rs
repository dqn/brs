use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayMode {
    Beat5K,
    Beat7K,
    Beat10K,
    Beat14K,
    PopN5K,
    PopN9K,
    Keyboard24K,
    Keyboard24KDouble,
}

impl PlayMode {
    /// Total number of key lanes (including scratch)
    pub fn key_count(self) -> usize {
        match self {
            Self::Beat5K => 6,
            Self::Beat7K => 8,
            Self::Beat10K => 12,
            Self::Beat14K => 16,
            Self::PopN5K => 5,
            Self::PopN9K => 9,
            Self::Keyboard24K => 26,
            Self::Keyboard24KDouble => 52,
        }
    }

    /// Number of players
    pub fn player_count(self) -> usize {
        match self {
            Self::Beat10K | Self::Beat14K | Self::Keyboard24KDouble => 2,
            _ => 1,
        }
    }

    /// Lane indices that are scratch keys (0-indexed)
    pub fn scratch_keys(self) -> &'static [usize] {
        match self {
            Self::Beat5K => &[5],
            Self::Beat7K => &[7],
            Self::Beat10K => &[5, 11],
            Self::Beat14K => &[7, 15],
            Self::PopN5K | Self::PopN9K => &[],
            Self::Keyboard24K => &[24, 25],
            Self::Keyboard24KDouble => &[24, 25, 50, 51],
        }
    }

    /// Detect play mode from #PLAYER value and key usage
    pub fn from_player_and_keys(player: i32, max_key_channel: usize) -> Self {
        match player {
            3 => {
                // Double play
                if max_key_channel > 8 {
                    Self::Beat14K
                } else {
                    Self::Beat10K
                }
            }
            _ => {
                // Single play: detect from channel usage
                if max_key_channel <= 5 {
                    Self::Beat5K
                } else if max_key_channel <= 8 {
                    Self::Beat7K
                } else {
                    Self::PopN9K
                }
            }
        }
    }
}

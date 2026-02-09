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

    /// Detect play mode from bmson mode_hint string
    pub fn from_mode_hint(hint: &str) -> Option<Self> {
        match hint {
            "beat-5k" => Some(Self::Beat5K),
            "beat-7k" => Some(Self::Beat7K),
            "beat-10k" => Some(Self::Beat10K),
            "beat-14k" => Some(Self::Beat14K),
            "popn-5k" => Some(Self::PopN5K),
            "popn-9k" => Some(Self::PopN9K),
            "keyboard-24k" => Some(Self::Keyboard24K),
            "keyboard-24k-double" => Some(Self::Keyboard24KDouble),
            _ => None,
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

    /// Returns true if the given lane index is a scratch key.
    ///
    /// Matches Java `Mode.isScratchKey(int key)`.
    pub fn is_scratch_key(self, lane: usize) -> bool {
        self.scratch_keys().contains(&lane)
    }

    /// Channel assignment table for 1P (index = channel - 0x11, value = lane or -1 for skip)
    pub fn channel_assign_1p(self) -> &'static [i8; 9] {
        match self {
            Self::Beat5K => &CHANNELASSIGN_BEAT5,
            Self::Beat7K => &CHANNELASSIGN_BEAT7,
            Self::PopN5K | Self::PopN9K => &CHANNELASSIGN_POPN,
            _ => &CHANNELASSIGN_BEAT7,
        }
    }

    /// Channel assignment table for 2P (index = channel - 0x21, value = lane or -1 for skip)
    pub fn channel_assign_2p(self) -> &'static [i8; 9] {
        match self {
            Self::Beat10K => &CHANNELASSIGN_BEAT5_2P,
            Self::Beat14K => &CHANNELASSIGN_BEAT7_2P,
            _ => &CHANNELASSIGN_BEAT7_2P,
        }
    }
}

// Channel assignment tables (Java: Mode.java)
// 1P: index = channel - 0x11, value = lane index (-1 = skip)
pub const CHANNELASSIGN_BEAT5: [i8; 9] = [0, 1, 2, 3, 4, 5, -1, -1, -1];
pub const CHANNELASSIGN_BEAT7: [i8; 9] = [0, 1, 2, 3, 4, 7, -1, 5, 6];
pub const CHANNELASSIGN_POPN: [i8; 9] = [0, 1, 2, 3, 4, -1, -1, -1, -1];

// 2P: index = channel - 0x21, value = lane index (-1 = skip)
pub const CHANNELASSIGN_BEAT5_2P: [i8; 9] = [6, 7, 8, 9, 10, 11, -1, -1, -1];
pub const CHANNELASSIGN_BEAT7_2P: [i8; 9] = [8, 9, 10, 11, 12, 15, -1, 13, 14];

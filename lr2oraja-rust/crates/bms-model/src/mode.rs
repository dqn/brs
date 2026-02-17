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

    /// Mode ID for database storage (matches Java Mode.id)
    pub fn mode_id(self) -> i32 {
        match self {
            Self::Beat5K => 5,
            Self::Beat7K => 7,
            Self::Beat10K => 10,
            Self::Beat14K => 14,
            Self::PopN5K | Self::PopN9K => 9,
            Self::Keyboard24K => 25,
            Self::Keyboard24KDouble => 50,
        }
    }

    /// Convert a mode ID back to PlayMode.
    ///
    /// Note: id=9 maps to PopN9K by default (PopN5K shares the same id).
    pub fn from_mode_id(id: i32) -> Option<Self> {
        match id {
            5 => Some(Self::Beat5K),
            7 => Some(Self::Beat7K),
            10 => Some(Self::Beat10K),
            14 => Some(Self::Beat14K),
            9 => Some(Self::PopN9K),
            25 => Some(Self::Keyboard24K),
            50 => Some(Self::Keyboard24KDouble),
            _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_count_all_modes() {
        assert_eq!(PlayMode::Beat5K.key_count(), 6);
        assert_eq!(PlayMode::Beat7K.key_count(), 8);
        assert_eq!(PlayMode::Beat10K.key_count(), 12);
        assert_eq!(PlayMode::Beat14K.key_count(), 16);
        assert_eq!(PlayMode::PopN5K.key_count(), 5);
        assert_eq!(PlayMode::PopN9K.key_count(), 9);
        assert_eq!(PlayMode::Keyboard24K.key_count(), 26);
        assert_eq!(PlayMode::Keyboard24KDouble.key_count(), 52);
    }

    #[test]
    fn player_count_double_play() {
        assert_eq!(PlayMode::Beat10K.player_count(), 2);
        assert_eq!(PlayMode::Beat14K.player_count(), 2);
        assert_eq!(PlayMode::Keyboard24KDouble.player_count(), 2);
    }

    #[test]
    fn player_count_single_play() {
        assert_eq!(PlayMode::Beat5K.player_count(), 1);
        assert_eq!(PlayMode::Beat7K.player_count(), 1);
        assert_eq!(PlayMode::PopN5K.player_count(), 1);
        assert_eq!(PlayMode::PopN9K.player_count(), 1);
        assert_eq!(PlayMode::Keyboard24K.player_count(), 1);
    }

    #[test]
    fn scratch_keys_all_modes() {
        assert_eq!(PlayMode::Beat5K.scratch_keys(), &[5]);
        assert_eq!(PlayMode::Beat7K.scratch_keys(), &[7]);
        assert_eq!(PlayMode::Beat10K.scratch_keys(), &[5, 11]);
        assert_eq!(PlayMode::Beat14K.scratch_keys(), &[7, 15]);
        assert_eq!(PlayMode::PopN5K.scratch_keys(), &[] as &[usize]);
        assert_eq!(PlayMode::PopN9K.scratch_keys(), &[] as &[usize]);
        assert_eq!(PlayMode::Keyboard24K.scratch_keys(), &[24, 25]);
        assert_eq!(
            PlayMode::Keyboard24KDouble.scratch_keys(),
            &[24, 25, 50, 51]
        );
    }

    #[test]
    fn from_mode_hint_valid() {
        assert_eq!(PlayMode::from_mode_hint("beat-5k"), Some(PlayMode::Beat5K));
        assert_eq!(PlayMode::from_mode_hint("beat-7k"), Some(PlayMode::Beat7K));
        assert_eq!(
            PlayMode::from_mode_hint("beat-10k"),
            Some(PlayMode::Beat10K)
        );
        assert_eq!(
            PlayMode::from_mode_hint("beat-14k"),
            Some(PlayMode::Beat14K)
        );
        assert_eq!(PlayMode::from_mode_hint("popn-5k"), Some(PlayMode::PopN5K));
        assert_eq!(PlayMode::from_mode_hint("popn-9k"), Some(PlayMode::PopN9K));
        assert_eq!(
            PlayMode::from_mode_hint("keyboard-24k"),
            Some(PlayMode::Keyboard24K)
        );
        assert_eq!(
            PlayMode::from_mode_hint("keyboard-24k-double"),
            Some(PlayMode::Keyboard24KDouble)
        );
    }

    #[test]
    fn from_mode_hint_invalid() {
        assert_eq!(PlayMode::from_mode_hint(""), None);
        assert_eq!(PlayMode::from_mode_hint("unknown"), None);
        assert_eq!(PlayMode::from_mode_hint("Beat-7k"), None); // case-sensitive
    }

    #[test]
    fn from_player_and_keys_double_play() {
        // player=3 with max_key > 8 -> Beat14K
        assert_eq!(PlayMode::from_player_and_keys(3, 9), PlayMode::Beat14K);
        assert_eq!(PlayMode::from_player_and_keys(3, 16), PlayMode::Beat14K);
        // player=3 with max_key <= 8 -> Beat10K
        assert_eq!(PlayMode::from_player_and_keys(3, 8), PlayMode::Beat10K);
        assert_eq!(PlayMode::from_player_and_keys(3, 1), PlayMode::Beat10K);
    }

    #[test]
    fn from_player_and_keys_single_play() {
        // max_key <= 5 -> Beat5K
        assert_eq!(PlayMode::from_player_and_keys(1, 5), PlayMode::Beat5K);
        assert_eq!(PlayMode::from_player_and_keys(1, 1), PlayMode::Beat5K);
        // max_key <= 8 -> Beat7K
        assert_eq!(PlayMode::from_player_and_keys(1, 6), PlayMode::Beat7K);
        assert_eq!(PlayMode::from_player_and_keys(1, 8), PlayMode::Beat7K);
        // max_key > 8 -> PopN9K
        assert_eq!(PlayMode::from_player_and_keys(1, 9), PlayMode::PopN9K);
        assert_eq!(PlayMode::from_player_and_keys(1, 24), PlayMode::PopN9K);
    }

    #[test]
    fn mode_id_round_trip() {
        // All modes except PopN5K can round-trip through mode_id
        let modes = [
            PlayMode::Beat5K,
            PlayMode::Beat7K,
            PlayMode::Beat10K,
            PlayMode::Beat14K,
            PlayMode::PopN9K,
            PlayMode::Keyboard24K,
            PlayMode::Keyboard24KDouble,
        ];
        for mode in modes {
            assert_eq!(
                PlayMode::from_mode_id(mode.mode_id()),
                Some(mode),
                "Round-trip failed for {:?}",
                mode
            );
        }
    }

    #[test]
    fn mode_id_popn5k_and_popn9k_share_id() {
        // Both PopN5K and PopN9K have mode_id 9
        assert_eq!(PlayMode::PopN5K.mode_id(), 9);
        assert_eq!(PlayMode::PopN9K.mode_id(), 9);
        // from_mode_id(9) returns PopN9K by convention
        assert_eq!(PlayMode::from_mode_id(9), Some(PlayMode::PopN9K));
    }

    #[test]
    fn from_mode_id_invalid() {
        assert_eq!(PlayMode::from_mode_id(0), None);
        assert_eq!(PlayMode::from_mode_id(99), None);
        assert_eq!(PlayMode::from_mode_id(-1), None);
    }

    #[test]
    fn is_scratch_key_beat7k() {
        // Lane 7 is scratch in Beat7K
        assert!(PlayMode::Beat7K.is_scratch_key(7));
        // Lane 0 is not scratch
        assert!(!PlayMode::Beat7K.is_scratch_key(0));
        assert!(!PlayMode::Beat7K.is_scratch_key(6));
    }

    #[test]
    fn is_scratch_key_popn9k_has_none() {
        for lane in 0..9 {
            assert!(
                !PlayMode::PopN9K.is_scratch_key(lane),
                "PopN9K lane {} should not be scratch",
                lane
            );
        }
    }

    #[test]
    fn channel_assign_1p_spot_checks() {
        // Beat7K 1P uses CHANNELASSIGN_BEAT7
        assert_eq!(PlayMode::Beat7K.channel_assign_1p(), &CHANNELASSIGN_BEAT7);
        // Beat5K 1P uses CHANNELASSIGN_BEAT5
        assert_eq!(PlayMode::Beat5K.channel_assign_1p(), &CHANNELASSIGN_BEAT5);
        // PopN modes use CHANNELASSIGN_POPN
        assert_eq!(PlayMode::PopN5K.channel_assign_1p(), &CHANNELASSIGN_POPN);
        assert_eq!(PlayMode::PopN9K.channel_assign_1p(), &CHANNELASSIGN_POPN);
        // Beat14K and Keyboard24K fall through to CHANNELASSIGN_BEAT7
        assert_eq!(PlayMode::Beat14K.channel_assign_1p(), &CHANNELASSIGN_BEAT7);
        assert_eq!(
            PlayMode::Keyboard24K.channel_assign_1p(),
            &CHANNELASSIGN_BEAT7
        );
    }

    #[test]
    fn channel_assign_2p_spot_checks() {
        // Beat10K 2P uses CHANNELASSIGN_BEAT5_2P
        assert_eq!(
            PlayMode::Beat10K.channel_assign_2p(),
            &CHANNELASSIGN_BEAT5_2P
        );
        // Beat14K 2P uses CHANNELASSIGN_BEAT7_2P
        assert_eq!(
            PlayMode::Beat14K.channel_assign_2p(),
            &CHANNELASSIGN_BEAT7_2P
        );
        // All other modes fall through to CHANNELASSIGN_BEAT7_2P
        assert_eq!(
            PlayMode::Beat7K.channel_assign_2p(),
            &CHANNELASSIGN_BEAT7_2P
        );
        assert_eq!(
            PlayMode::Beat5K.channel_assign_2p(),
            &CHANNELASSIGN_BEAT7_2P
        );
    }
}

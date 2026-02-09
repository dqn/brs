use serde::{Deserialize, Serialize};

/// Skin type identifier.
///
/// Each variant has an ID, name, optional play mode ID, and battle flag.
/// This enum is intentionally independent of `bms-model` to avoid circular dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinType {
    #[serde(rename = "PLAY_7KEYS")]
    Play7Keys,
    #[serde(rename = "PLAY_5KEYS")]
    Play5Keys,
    #[serde(rename = "PLAY_14KEYS")]
    Play14Keys,
    #[serde(rename = "PLAY_10KEYS")]
    Play10Keys,
    #[serde(rename = "PLAY_9KEYS")]
    Play9Keys,
    #[serde(rename = "MUSIC_SELECT")]
    MusicSelect,
    #[serde(rename = "DECIDE")]
    Decide,
    #[serde(rename = "RESULT")]
    Result,
    #[serde(rename = "KEY_CONFIG")]
    KeyConfig,
    #[serde(rename = "SKIN_SELECT")]
    SkinSelect,
    #[serde(rename = "SOUND_SET")]
    SoundSet,
    #[serde(rename = "THEME")]
    Theme,
    #[serde(rename = "PLAY_7KEYS_BATTLE")]
    Play7KeysBattle,
    #[serde(rename = "PLAY_5KEYS_BATTLE")]
    Play5KeysBattle,
    #[serde(rename = "PLAY_9KEYS_BATTLE")]
    Play9KeysBattle,
    #[serde(rename = "COURSE_RESULT")]
    CourseResult,
    #[serde(rename = "PLAY_24KEYS")]
    Play24Keys,
    #[serde(rename = "PLAY_24KEYS_DOUBLE")]
    Play24KeysDouble,
    #[serde(rename = "PLAY_24KEYS_BATTLE")]
    Play24KeysBattle,
}

impl SkinType {
    /// Returns the numeric ID for this skin type.
    pub fn id(self) -> i32 {
        match self {
            Self::Play7Keys => 0,
            Self::Play5Keys => 1,
            Self::Play14Keys => 2,
            Self::Play10Keys => 3,
            Self::Play9Keys => 4,
            Self::MusicSelect => 5,
            Self::Decide => 6,
            Self::Result => 7,
            Self::KeyConfig => 8,
            Self::SkinSelect => 9,
            Self::SoundSet => 10,
            Self::Theme => 11,
            Self::Play7KeysBattle => 12,
            Self::Play5KeysBattle => 13,
            Self::Play9KeysBattle => 14,
            Self::CourseResult => 15,
            Self::Play24Keys => 16,
            Self::Play24KeysDouble => 17,
            Self::Play24KeysBattle => 18,
        }
    }

    /// Returns the display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Play7Keys => "7KEYS",
            Self::Play5Keys => "5KEYS",
            Self::Play14Keys => "14KEYS",
            Self::Play10Keys => "10KEYS",
            Self::Play9Keys => "9KEYS",
            Self::MusicSelect => "MUSIC SELECT",
            Self::Decide => "DECIDE",
            Self::Result => "RESULT",
            Self::KeyConfig => "KEY CONFIG",
            Self::SkinSelect => "SKIN SELECT",
            Self::SoundSet => "SOUND SET",
            Self::Theme => "THEME",
            Self::Play7KeysBattle => "7KEYS BATTLE",
            Self::Play5KeysBattle => "5KEYS BATTLE",
            Self::Play9KeysBattle => "9KEYS BATTLE",
            Self::CourseResult => "COURSE RESULT",
            Self::Play24Keys => "24KEYS",
            Self::Play24KeysDouble => "24KEYS DOUBLE",
            Self::Play24KeysBattle => "24KEYS BATTLE",
        }
    }

    /// Returns the play mode ID if this is a play skin type.
    ///
    /// Maps to `bms-model` Mode IDs without depending on it:
    /// - 7K -> 7, 5K -> 5, 14K -> 14, 10K -> 10, 9K -> 9, 24K -> 25, 24K_DOUBLE -> 50
    pub fn mode_id(self) -> Option<i32> {
        match self {
            Self::Play7Keys | Self::Play7KeysBattle => Some(7),
            Self::Play5Keys | Self::Play5KeysBattle => Some(5),
            Self::Play14Keys => Some(14),
            Self::Play10Keys => Some(10),
            Self::Play9Keys | Self::Play9KeysBattle => Some(9),
            Self::Play24Keys | Self::Play24KeysBattle => Some(25),
            Self::Play24KeysDouble => Some(50),
            _ => None,
        }
    }

    /// Returns true if this skin type is for play screens.
    pub fn is_play(self) -> bool {
        self.mode_id().is_some()
    }

    /// Returns true if this skin type is for battle mode.
    pub fn is_battle(self) -> bool {
        matches!(
            self,
            Self::Play7KeysBattle
                | Self::Play5KeysBattle
                | Self::Play9KeysBattle
                | Self::Play24KeysBattle
        )
    }

    /// Looks up a skin type by its numeric ID.
    pub fn from_id(id: i32) -> Option<Self> {
        Self::all().into_iter().find(|t| t.id() == id)
    }

    /// Returns the maximum skin type ID.
    pub fn max_id() -> i32 {
        18
    }

    /// Returns all skin type variants.
    fn all() -> [Self; 19] {
        [
            Self::Play7Keys,
            Self::Play5Keys,
            Self::Play14Keys,
            Self::Play10Keys,
            Self::Play9Keys,
            Self::MusicSelect,
            Self::Decide,
            Self::Result,
            Self::KeyConfig,
            Self::SkinSelect,
            Self::SoundSet,
            Self::Theme,
            Self::Play7KeysBattle,
            Self::Play5KeysBattle,
            Self::Play9KeysBattle,
            Self::CourseResult,
            Self::Play24Keys,
            Self::Play24KeysDouble,
            Self::Play24KeysBattle,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ids_sequential() {
        for (i, st) in SkinType::all().iter().enumerate() {
            assert_eq!(st.id(), i as i32);
        }
    }

    #[test]
    fn test_max_id() {
        assert_eq!(SkinType::max_id(), 18);
    }

    #[test]
    fn test_from_id() {
        assert_eq!(SkinType::from_id(0), Some(SkinType::Play7Keys));
        assert_eq!(SkinType::from_id(5), Some(SkinType::MusicSelect));
        assert_eq!(SkinType::from_id(18), Some(SkinType::Play24KeysBattle));
        assert_eq!(SkinType::from_id(19), None);
        assert_eq!(SkinType::from_id(-1), None);
    }

    #[test]
    fn test_mode_id() {
        assert_eq!(SkinType::Play7Keys.mode_id(), Some(7));
        assert_eq!(SkinType::Play5Keys.mode_id(), Some(5));
        assert_eq!(SkinType::Play14Keys.mode_id(), Some(14));
        assert_eq!(SkinType::Play10Keys.mode_id(), Some(10));
        assert_eq!(SkinType::Play9Keys.mode_id(), Some(9));
        assert_eq!(SkinType::Play24Keys.mode_id(), Some(25));
        assert_eq!(SkinType::Play24KeysDouble.mode_id(), Some(50));
        assert_eq!(SkinType::MusicSelect.mode_id(), None);
        assert_eq!(SkinType::Decide.mode_id(), None);
        // Battle variants share mode_id with non-battle
        assert_eq!(SkinType::Play7KeysBattle.mode_id(), Some(7));
        assert_eq!(SkinType::Play5KeysBattle.mode_id(), Some(5));
        assert_eq!(SkinType::Play9KeysBattle.mode_id(), Some(9));
        assert_eq!(SkinType::Play24KeysBattle.mode_id(), Some(25));
    }

    #[test]
    fn test_is_play() {
        assert!(SkinType::Play7Keys.is_play());
        assert!(SkinType::Play7KeysBattle.is_play());
        assert!(!SkinType::MusicSelect.is_play());
        assert!(!SkinType::Decide.is_play());
        assert!(!SkinType::Theme.is_play());
    }

    #[test]
    fn test_is_battle() {
        assert!(SkinType::Play7KeysBattle.is_battle());
        assert!(SkinType::Play5KeysBattle.is_battle());
        assert!(SkinType::Play9KeysBattle.is_battle());
        assert!(SkinType::Play24KeysBattle.is_battle());
        assert!(!SkinType::Play7Keys.is_battle());
        assert!(!SkinType::MusicSelect.is_battle());
    }

    #[test]
    fn test_serde_round_trip() {
        let st = SkinType::Play7Keys;
        let json = serde_json::to_string(&st).unwrap();
        assert_eq!(json, "\"PLAY_7KEYS\"");
        let back: SkinType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, st);
    }

    #[test]
    fn test_names() {
        assert_eq!(SkinType::Play7Keys.name(), "7KEYS");
        assert_eq!(SkinType::MusicSelect.name(), "MUSIC SELECT");
        assert_eq!(SkinType::Play24KeysDouble.name(), "24KEYS DOUBLE");
    }
}

/// Skin type for different game modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkinType {
    /// 7-key play skin (type = 0)
    Play7 = 0,
    /// 5-key play skin (type = 1)
    Play5 = 1,
    /// 14-key play skin (type = 2)
    Play14 = 2,
    /// 10-key play skin (type = 3)
    Play10 = 3,
    /// 9-key play skin (type = 4)
    Play9 = 4,
    /// Music select skin (type = 5)
    Select = 5,
    /// Decide skin (type = 6)
    Decide = 6,
    /// Result skin (type = 7)
    Result = 7,
    /// Key configuration skin (type = 8)
    KeyConfig = 8,
    /// Skin configuration skin (type = 9)
    SkinConfig = 9,
    /// Course result skin (type = 10)
    CourseResult = 10,
}

impl SkinType {
    /// Parse skin type from integer value.
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Play7),
            1 => Some(Self::Play5),
            2 => Some(Self::Play14),
            3 => Some(Self::Play10),
            4 => Some(Self::Play9),
            5 => Some(Self::Select),
            6 => Some(Self::Decide),
            7 => Some(Self::Result),
            8 => Some(Self::KeyConfig),
            9 => Some(Self::SkinConfig),
            10 => Some(Self::CourseResult),
            _ => None,
        }
    }

    /// Check if this is a play skin type.
    pub fn is_play(&self) -> bool {
        matches!(
            self,
            Self::Play7 | Self::Play5 | Self::Play14 | Self::Play10 | Self::Play9
        )
    }
}

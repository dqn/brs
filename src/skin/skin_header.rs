use std::path::PathBuf;

/// Skin type (play mode / scene).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkinType {
    Play7Keys = 0,
    Play5Keys = 1,
    Play14Keys = 2,
    Play10Keys = 3,
    Play9Keys = 4,
    MusicSelect = 5,
    Decide = 6,
    Result = 7,
    KeyConfig = 8,
    SkinSelect = 9,
    SoundSet = 10,
    Theme = 11,
    Play7KeysBattle = 12,
    Play5KeysBattle = 13,
    Play9KeysBattle = 14,
    CourseResult = 15,
    Play24Keys = 16,
    Play24KeysDouble = 17,
    Play24KeysBattle = 18,
}

impl SkinType {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Play7Keys),
            1 => Some(Self::Play5Keys),
            2 => Some(Self::Play14Keys),
            3 => Some(Self::Play10Keys),
            4 => Some(Self::Play9Keys),
            5 => Some(Self::MusicSelect),
            6 => Some(Self::Decide),
            7 => Some(Self::Result),
            8 => Some(Self::KeyConfig),
            9 => Some(Self::SkinSelect),
            10 => Some(Self::SoundSet),
            11 => Some(Self::Theme),
            12 => Some(Self::Play7KeysBattle),
            13 => Some(Self::Play5KeysBattle),
            14 => Some(Self::Play9KeysBattle),
            15 => Some(Self::CourseResult),
            16 => Some(Self::Play24Keys),
            17 => Some(Self::Play24KeysDouble),
            18 => Some(Self::Play24KeysBattle),
            _ => None,
        }
    }

    pub fn is_play(&self) -> bool {
        matches!(
            self,
            Self::Play7Keys
                | Self::Play5Keys
                | Self::Play14Keys
                | Self::Play10Keys
                | Self::Play9Keys
                | Self::Play7KeysBattle
                | Self::Play5KeysBattle
                | Self::Play9KeysBattle
                | Self::Play24Keys
                | Self::Play24KeysDouble
                | Self::Play24KeysBattle
        )
    }
}

/// User-customizable option for skin configuration.
#[derive(Debug, Clone)]
pub struct CustomOption {
    pub name: String,
    pub option_ids: Vec<i32>,
    pub option_names: Vec<String>,
    pub default_name: Option<String>,
    pub selected_index: i32,
}

/// User-customizable file path.
#[derive(Debug, Clone)]
pub struct CustomFile {
    pub name: String,
    pub path: String,
    pub default_name: Option<String>,
    pub selected_filename: Option<String>,
}

/// User-customizable offset.
#[derive(Debug, Clone)]
pub struct CustomOffset {
    pub name: String,
    pub id: i32,
    pub x: bool,
    pub y: bool,
    pub w: bool,
    pub h: bool,
    pub r: bool,
    pub a: bool,
}

/// Skin header metadata.
#[derive(Debug, Clone)]
pub struct SkinHeader {
    pub skin_type: SkinType,
    pub name: String,
    pub author: String,
    pub path: PathBuf,
    /// Source resolution width.
    pub src_width: u32,
    /// Source resolution height.
    pub src_height: u32,
    /// Scene time in ms.
    pub scene: i32,
    /// Input acceptance start time in ms.
    pub input: i32,
    /// Fadeout time in ms.
    pub fadeout: i32,
    /// Custom options.
    pub options: Vec<CustomOption>,
    /// Custom file paths.
    pub files: Vec<CustomFile>,
    /// Custom offsets.
    pub offsets: Vec<CustomOffset>,
    /// Load end time for play skins.
    pub load_end: i32,
    /// Play start time.
    pub play_start: i32,
    /// Close time.
    pub close: i32,
}

impl Default for SkinHeader {
    fn default() -> Self {
        Self {
            skin_type: SkinType::Play7Keys,
            name: String::new(),
            author: String::new(),
            path: PathBuf::new(),
            src_width: 1920,
            src_height: 1080,
            scene: 3_600_000,
            input: 0,
            fadeout: 0,
            options: Vec::new(),
            files: Vec::new(),
            offsets: Vec::new(),
            load_end: 0,
            play_start: 0,
            close: 0,
        }
    }
}

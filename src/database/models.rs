use std::path::PathBuf;

/// Play mode for BMS chart (beatoraja compatible).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum Mode {
    Beat5K = 5,
    #[default]
    Beat7K = 7,
    Beat10K = 10,
    Beat14K = 14,
    PopN5K = 25,
    PopN9K = 29,
}

impl Mode {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            5 => Some(Self::Beat5K),
            7 => Some(Self::Beat7K),
            10 => Some(Self::Beat10K),
            14 => Some(Self::Beat14K),
            25 => Some(Self::PopN5K),
            29 => Some(Self::PopN9K),
            _ => None,
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Clear type (beatoraja compatible, 11 levels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(i32)]
pub enum ClearType {
    #[default]
    NoPlay = 0,
    Failed = 1,
    AssistEasy = 2,
    LightAssistEasy = 3,
    Easy = 4,
    Normal = 5,
    Hard = 6,
    ExHard = 7,
    FullCombo = 8,
    Perfect = 9,
    Max = 10,
}

impl ClearType {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::NoPlay),
            1 => Some(Self::Failed),
            2 => Some(Self::AssistEasy),
            3 => Some(Self::LightAssistEasy),
            4 => Some(Self::Easy),
            5 => Some(Self::Normal),
            6 => Some(Self::Hard),
            7 => Some(Self::ExHard),
            8 => Some(Self::FullCombo),
            9 => Some(Self::Perfect),
            10 => Some(Self::Max),
            _ => None,
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Returns true if this clear type represents a successful clear.
    pub fn is_cleared(self) -> bool {
        self >= Self::AssistEasy
    }
}

/// Song metadata stored in song.db.
#[derive(Debug, Clone)]
pub struct SongData {
    /// SHA256 hash of the BMS file.
    pub sha256: String,
    /// MD5 hash of the BMS file (for beatoraja compatibility).
    pub md5: String,
    /// Absolute path to the BMS file.
    pub path: PathBuf,
    /// Folder path (relative to the BMS root).
    pub folder: String,
    /// Song title.
    pub title: String,
    /// Subtitle.
    pub subtitle: String,
    /// Artist name.
    pub artist: String,
    /// Sub-artist name.
    pub subartist: String,
    /// Genre.
    pub genre: String,
    /// Play mode (5K, 7K, etc.).
    pub mode: Mode,
    /// Difficulty level.
    pub level: i32,
    /// Difficulty type (1=BEGINNER, 2=NORMAL, 3=HYPER, 4=ANOTHER, 5=INSANE).
    pub difficulty: i32,
    /// Maximum BPM.
    pub max_bpm: i32,
    /// Minimum BPM.
    pub min_bpm: i32,
    /// Total note count.
    pub notes: i32,
    /// File modification timestamp (Unix seconds).
    pub date: i64,
    /// Timestamp when added to database (Unix seconds).
    pub add_date: i64,
}

impl SongData {
    pub fn new(sha256: String, path: PathBuf) -> Self {
        Self {
            sha256,
            md5: String::new(),
            path,
            folder: String::new(),
            title: String::new(),
            subtitle: String::new(),
            artist: String::new(),
            subartist: String::new(),
            genre: String::new(),
            mode: Mode::default(),
            level: 0,
            difficulty: 0,
            max_bpm: 0,
            min_bpm: 0,
            notes: 0,
            date: 0,
            add_date: 0,
        }
    }
}

/// Score data stored in score.db.
#[derive(Debug, Clone)]
pub struct ScoreData {
    /// SHA256 hash of the BMS file.
    pub sha256: String,
    /// LN mode (0=normal, 1=LN, 2=CN, 3=HCN).
    pub mode: i32,
    /// Clear type.
    pub clear: ClearType,
    /// EX score (PG*2 + GR).
    pub ex_score: i32,
    /// Maximum combo achieved.
    pub max_combo: i32,
    /// Minimum BP (bad + poor + miss).
    pub min_bp: i32,
    /// Perfect Great count.
    pub pg: i32,
    /// Great count.
    pub gr: i32,
    /// Good count.
    pub gd: i32,
    /// Bad count.
    pub bd: i32,
    /// Poor count.
    pub pr: i32,
    /// Miss count.
    pub ms: i32,
    /// Total note count at time of play.
    pub notes: i32,
    /// Total play count.
    pub play_count: i32,
    /// Successful clear count.
    pub clear_count: i32,
    /// Last play timestamp (Unix seconds).
    pub date: i64,
}

impl ScoreData {
    pub fn new(sha256: String) -> Self {
        Self {
            sha256,
            mode: 0,
            clear: ClearType::default(),
            ex_score: 0,
            max_combo: 0,
            min_bp: i32::MAX,
            pg: 0,
            gr: 0,
            gd: 0,
            bd: 0,
            pr: 0,
            ms: 0,
            notes: 0,
            play_count: 0,
            clear_count: 0,
            date: 0,
        }
    }

    /// Calculate BP from judge counts.
    pub fn bp(&self) -> i32 {
        self.bd + self.pr + self.ms
    }
}

/// Result of a folder scan operation.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// Number of newly added songs.
    pub added: usize,
    /// Number of updated songs.
    pub updated: usize,
    /// Number of deleted songs.
    pub deleted: usize,
    /// Number of unchanged songs.
    pub unchanged: usize,
    /// Errors encountered during scan.
    pub errors: Vec<(PathBuf, String)>,
}

impl ScanResult {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total number of songs processed.
    pub fn total(&self) -> usize {
        self.added + self.updated + self.deleted + self.unchanged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_type_ordering() {
        assert!(ClearType::Max > ClearType::Perfect);
        assert!(ClearType::Perfect > ClearType::FullCombo);
        assert!(ClearType::FullCombo > ClearType::ExHard);
        assert!(ClearType::NoPlay < ClearType::Failed);
    }

    #[test]
    fn test_clear_type_is_cleared() {
        assert!(!ClearType::NoPlay.is_cleared());
        assert!(!ClearType::Failed.is_cleared());
        assert!(ClearType::AssistEasy.is_cleared());
        assert!(ClearType::Normal.is_cleared());
        assert!(ClearType::Max.is_cleared());
    }

    #[test]
    fn test_mode_conversion() {
        assert_eq!(Mode::Beat7K.as_i32(), 7);
        assert_eq!(Mode::from_i32(7), Some(Mode::Beat7K));
        assert_eq!(Mode::from_i32(999), None);
    }

    #[test]
    fn test_clear_type_conversion() {
        assert_eq!(ClearType::Normal.as_i32(), 5);
        assert_eq!(ClearType::from_i32(5), Some(ClearType::Normal));
        assert_eq!(ClearType::from_i32(99), None);
    }
}

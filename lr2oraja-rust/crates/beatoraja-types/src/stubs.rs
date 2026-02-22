// Phase 5+ stubs — types from downstream crates that beatoraja-core cannot import
// due to circular dependency constraints. These will be replaced if/when the
// dependency graph is restructured (e.g., extracting shared types into a common crate).

// ---------------------------------------------------------------------------
// beatoraja-play stubs
// ---------------------------------------------------------------------------

// GrooveGauge moved to beatoraja-types/src/groove_gauge.rs (Phase 15b)
pub use crate::groove_gauge::GrooveGauge;

/// Judge algorithm enum.
///
/// Translated from: bms.player.beatoraja.play.JudgeAlgorithm
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JudgeAlgorithm {
    Combo,
    Duration,
    Lowest,
    Score,
}

impl JudgeAlgorithm {
    pub const DEFAULT_ALGORITHM: &'static [JudgeAlgorithm] = &[
        JudgeAlgorithm::Combo,
        JudgeAlgorithm::Duration,
        JudgeAlgorithm::Lowest,
    ];

    pub fn name(&self) -> &str {
        match self {
            JudgeAlgorithm::Combo => "Combo",
            JudgeAlgorithm::Duration => "Duration",
            JudgeAlgorithm::Lowest => "Lowest",
            JudgeAlgorithm::Score => "Score",
        }
    }

    pub fn get_index(name: &str) -> i32 {
        for (i, v) in Self::values().iter().enumerate() {
            if v.name() == name {
                return i as i32;
            }
        }
        -1
    }

    pub fn values() -> &'static [JudgeAlgorithm] {
        &[
            JudgeAlgorithm::Combo,
            JudgeAlgorithm::Duration,
            JudgeAlgorithm::Lowest,
            JudgeAlgorithm::Score,
        ]
    }
}

/// Player rule enum.
///
/// Translated from: bms.player.beatoraja.play.BMSPlayerRule
///
/// In Java, each variant carries GaugeProperty, JudgeProperty, and Mode[] fields.
/// Since beatoraja-types cannot depend on those crate-specific types, this enum
/// only captures the variant names. Cross-crate logic uses these variants as keys
/// to look up the corresponding properties.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BMSPlayerRule {
    Beatoraja5,
    Beatoraja7,
    Beatoraja9,
    Beatoraja24,
    BeatorajaOther,
    LR2,
    Default,
}

// ---------------------------------------------------------------------------
// beatoraja-skin: SkinType moved to beatoraja-types/src/skin_type.rs
// ---------------------------------------------------------------------------

pub use crate::skin_type::SkinType;

// ---------------------------------------------------------------------------
// beatoraja-select stubs
// ---------------------------------------------------------------------------

/// Bar sorting algorithm enum.
///
/// Translated from: bms.player.beatoraja.select.BarSorter
///
/// This is a lightweight copy of the enum defined in beatoraja-select.
/// beatoraja-types cannot depend on beatoraja-select, so the sorting logic
/// is not included here — only the enum variants, name mapping, and default list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarSorter {
    Title,
    Artist,
    Bpm,
    Length,
    Level,
    Clear,
    Score,
    MissCount,
    Duration,
    LastUpdate,
    RivalCompareClear,
    RivalCompareScore,
}

impl BarSorter {
    pub const DEFAULT_SORTER: &'static [BarSorter] = &[
        BarSorter::Title,
        BarSorter::Artist,
        BarSorter::Bpm,
        BarSorter::Length,
        BarSorter::Level,
        BarSorter::Clear,
        BarSorter::Score,
        BarSorter::MissCount,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            BarSorter::Title => "TITLE",
            BarSorter::Artist => "ARTIST",
            BarSorter::Bpm => "BPM",
            BarSorter::Length => "LENGTH",
            BarSorter::Level => "LEVEL",
            BarSorter::Clear => "CLEAR",
            BarSorter::Score => "SCORE",
            BarSorter::MissCount => "MISSCOUNT",
            BarSorter::Duration => "DURATION",
            BarSorter::LastUpdate => "LASTUPDATE",
            BarSorter::RivalCompareClear => "RIVALCOMPARE_CLEAR",
            BarSorter::RivalCompareScore => "RIVALCOMPARE_SCORE",
        }
    }
}

// ---------------------------------------------------------------------------
// beatoraja-pattern stubs
// ---------------------------------------------------------------------------

/// Scroll speed modifier mode.
///
/// Translated from: bms.player.beatoraja.pattern.ScrollSpeedModifier.Mode
pub mod scroll_speed_modifier {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        Remove,
        Add,
    }

    impl Mode {
        pub fn values() -> &'static [Mode] {
            &[Mode::Remove, Mode::Add]
        }
    }
}

/// Long note modifier mode.
///
/// Translated from: bms.player.beatoraja.pattern.LongNoteModifier.Mode
pub mod long_note_modifier {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        Remove,
        AddLn,
        AddCn,
        AddHcn,
        AddAll,
    }

    impl Mode {
        pub fn values() -> &'static [Mode] {
            &[
                Mode::Remove,
                Mode::AddLn,
                Mode::AddCn,
                Mode::AddHcn,
                Mode::AddAll,
            ]
        }
    }
}

/// Mine note modifier mode.
///
/// Translated from: bms.player.beatoraja.pattern.MineNoteModifier.Mode
pub mod mine_note_modifier {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        Remove,
        AddRandom,
        AddNear,
        AddBlank,
    }

    impl Mode {
        pub fn values() -> &'static [Mode] {
            &[Mode::Remove, Mode::AddRandom, Mode::AddNear, Mode::AddBlank]
        }
    }
}

// ---------------------------------------------------------------------------
// beatoraja-ir stubs
// ---------------------------------------------------------------------------

/// IR connection manager stub.
///
/// Translated from: bms.player.beatoraja.ir.IRConnectionManager
///
/// beatoraja-types cannot depend on beatoraja-ir, so this provides the
/// same static API surface. The real implementation in beatoraja-ir uses
/// a registry of IRConnection trait objects.
pub struct IRConnectionManager;

impl IRConnectionManager {
    /// Get all available IR connection names.
    pub fn get_all_available_ir_connection_name() -> Vec<String> {
        vec![]
    }

    /// Check if an IR connection class exists for the given name.
    /// Returns Some(()) if found, None if empty/not found.
    pub fn get_ir_connection_class(name: &str) -> Option<()> {
        if name.is_empty() { None } else { Some(()) }
    }

    /// Get the home URL for an IR by name.
    pub fn get_home_url(_name: &str) -> Option<String> {
        None
    }
}

// ---------------------------------------------------------------------------
// beatoraja-input stubs (incompatible field layout with beatoraja-input crate)
// ---------------------------------------------------------------------------

/// Input device type enum for serialization.
///
/// Translated from: bms.player.beatoraja.input.BMSPlayerInputDevice.Type
///
/// Java has: KEYBOARD, BM_CONTROLLER, MIDI (3 variants).
/// This is a serialization-focused copy; the runtime version is in beatoraja-input.
pub mod bms_player_input_device {
    #[allow(non_camel_case_types)]
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum Type {
        KEYBOARD,
        BM_CONTROLLER,
        MIDI,
    }
}

/// Key input log for serialization.
///
/// Translated from: bms.player.beatoraja.input.KeyInputLog
///
/// This is a serialization-focused DTO with pub fields, used in ReplayData.
/// The runtime version in beatoraja-input uses private fields with getters.
/// Java field `presstime` is mapped to `time` for JSON compatibility.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct KeyInputLog {
    /// Key press time (us). Maps to Java `presstime` field.
    pub time: i64,
    /// Key code
    pub keycode: i32,
    /// Key pressed/released
    pub pressed: bool,
}

impl KeyInputLog {
    /// Validate the key input log.
    ///
    /// Translated from: KeyInputLog.validate()
    /// Java: return presstime >= 0 && keycode >= 0;
    pub fn validate(&self) -> bool {
        self.time >= 0 && self.keycode >= 0
    }
}

// ---------------------------------------------------------------------------
// beatoraja-pattern stubs (incompatible field layout with beatoraja-pattern crate)
// ---------------------------------------------------------------------------

/// Pattern modification log for serialization.
///
/// Translated from: bms.player.beatoraja.pattern.PatternModifyLog
///
/// This is a serialization-focused DTO with pub fields, used in ReplayData.
/// The runtime version in beatoraja-pattern implements the Validatable trait.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PatternModifyLog {
    /// Target TimeLine section
    #[serde(default = "default_section")]
    pub section: f64,
    /// Lane remap array for each lane in the target TimeLine
    pub modify: Option<Vec<i32>>,
}

fn default_section() -> f64 {
    -1.0
}

impl Default for PatternModifyLog {
    fn default() -> Self {
        PatternModifyLog {
            section: -1.0,
            modify: None,
        }
    }
}

impl PatternModifyLog {
    pub fn new(section: f64, modify: Vec<i32>) -> Self {
        PatternModifyLog {
            section,
            modify: Some(modify),
        }
    }

    /// Validate the pattern modification log.
    ///
    /// Translated from: PatternModifyLog.validate()
    /// Java: return section >= 0 && modify != null;
    pub fn validate(&self) -> bool {
        self.section >= 0.0 && self.modify.is_some()
    }
}

// ---------------------------------------------------------------------------
// beatoraja-song stubs — SongData moved to beatoraja-types/src/song_data.rs
// ---------------------------------------------------------------------------

pub use crate::song_data::SongData;

#[cfg(test)]
mod tests {
    use super::*;

    // -- JudgeAlgorithm --

    #[test]
    fn judge_algorithm_values_has_4_variants() {
        assert_eq!(JudgeAlgorithm::values().len(), 4);
    }

    #[test]
    fn judge_algorithm_default_has_3_variants() {
        assert_eq!(JudgeAlgorithm::DEFAULT_ALGORITHM.len(), 3);
    }

    #[test]
    fn judge_algorithm_name_matches_java() {
        assert_eq!(JudgeAlgorithm::Combo.name(), "Combo");
        assert_eq!(JudgeAlgorithm::Duration.name(), "Duration");
        assert_eq!(JudgeAlgorithm::Lowest.name(), "Lowest");
        assert_eq!(JudgeAlgorithm::Score.name(), "Score");
    }

    #[test]
    fn judge_algorithm_get_index() {
        assert_eq!(JudgeAlgorithm::get_index("Combo"), 0);
        assert_eq!(JudgeAlgorithm::get_index("Duration"), 1);
        assert_eq!(JudgeAlgorithm::get_index("Lowest"), 2);
        assert_eq!(JudgeAlgorithm::get_index("Score"), 3);
        assert_eq!(JudgeAlgorithm::get_index("Unknown"), -1);
    }

    #[test]
    fn judge_algorithm_serde_roundtrip() {
        let algo = JudgeAlgorithm::Score;
        let json = serde_json::to_string(&algo).unwrap();
        let deserialized: JudgeAlgorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, JudgeAlgorithm::Score);
    }

    // -- BMSPlayerRule --

    #[test]
    fn bms_player_rule_has_7_variants() {
        let rules = [
            BMSPlayerRule::Beatoraja5,
            BMSPlayerRule::Beatoraja7,
            BMSPlayerRule::Beatoraja9,
            BMSPlayerRule::Beatoraja24,
            BMSPlayerRule::BeatorajaOther,
            BMSPlayerRule::LR2,
            BMSPlayerRule::Default,
        ];
        assert_eq!(rules.len(), 7);
    }

    #[test]
    fn bms_player_rule_serde_roundtrip() {
        let rule = BMSPlayerRule::LR2;
        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: BMSPlayerRule = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, BMSPlayerRule::LR2);
    }

    // -- BarSorter --

    #[test]
    fn bar_sorter_default_has_8_entries() {
        assert_eq!(BarSorter::DEFAULT_SORTER.len(), 8);
    }

    #[test]
    fn bar_sorter_names_match_java() {
        assert_eq!(BarSorter::Title.name(), "TITLE");
        assert_eq!(BarSorter::Artist.name(), "ARTIST");
        assert_eq!(BarSorter::Bpm.name(), "BPM");
        assert_eq!(BarSorter::Length.name(), "LENGTH");
        assert_eq!(BarSorter::Level.name(), "LEVEL");
        assert_eq!(BarSorter::Clear.name(), "CLEAR");
        assert_eq!(BarSorter::Score.name(), "SCORE");
        assert_eq!(BarSorter::MissCount.name(), "MISSCOUNT");
        assert_eq!(BarSorter::Duration.name(), "DURATION");
        assert_eq!(BarSorter::LastUpdate.name(), "LASTUPDATE");
        assert_eq!(BarSorter::RivalCompareClear.name(), "RIVALCOMPARE_CLEAR");
        assert_eq!(BarSorter::RivalCompareScore.name(), "RIVALCOMPARE_SCORE");
    }

    // -- Modifier modes --

    #[test]
    fn scroll_speed_modifier_values_count() {
        assert_eq!(scroll_speed_modifier::Mode::values().len(), 2);
    }

    #[test]
    fn long_note_modifier_values_count() {
        assert_eq!(long_note_modifier::Mode::values().len(), 5);
    }

    #[test]
    fn mine_note_modifier_values_count() {
        assert_eq!(mine_note_modifier::Mode::values().len(), 4);
    }

    // -- bms_player_input_device::Type --

    #[test]
    fn input_device_type_has_3_variants() {
        let types = [
            bms_player_input_device::Type::KEYBOARD,
            bms_player_input_device::Type::BM_CONTROLLER,
            bms_player_input_device::Type::MIDI,
        ];
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn input_device_type_serde_roundtrip() {
        let t = bms_player_input_device::Type::BM_CONTROLLER;
        let json = serde_json::to_string(&t).unwrap();
        let deserialized: bms_player_input_device::Type = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, bms_player_input_device::Type::BM_CONTROLLER);
    }

    // -- KeyInputLog --

    #[test]
    fn key_input_log_validate_positive() {
        let log = KeyInputLog {
            time: 1000,
            keycode: 0,
            pressed: true,
        };
        assert!(log.validate());
    }

    #[test]
    fn key_input_log_validate_negative_time() {
        let log = KeyInputLog {
            time: -1,
            keycode: 0,
            pressed: true,
        };
        assert!(!log.validate());
    }

    #[test]
    fn key_input_log_validate_negative_keycode() {
        let log = KeyInputLog {
            time: 0,
            keycode: -1,
            pressed: true,
        };
        assert!(!log.validate());
    }

    // -- PatternModifyLog --

    #[test]
    fn pattern_modify_log_default() {
        let log = PatternModifyLog::default();
        assert_eq!(log.section, -1.0);
        assert!(log.modify.is_none());
        assert!(!log.validate());
    }

    #[test]
    fn pattern_modify_log_new_valid() {
        let log = PatternModifyLog::new(1.0, vec![0, 2, 1, 3]);
        assert_eq!(log.section, 1.0);
        assert_eq!(log.modify, Some(vec![0, 2, 1, 3]));
        assert!(log.validate());
    }

    #[test]
    fn pattern_modify_log_validate_negative_section() {
        let log = PatternModifyLog::new(-1.0, vec![0, 1]);
        assert!(!log.validate());
    }

    #[test]
    fn pattern_modify_log_validate_none_modify() {
        let log = PatternModifyLog {
            section: 1.0,
            modify: None,
        };
        assert!(!log.validate());
    }

    #[test]
    fn pattern_modify_log_serde_roundtrip() {
        let log = PatternModifyLog::new(2.5, vec![1, 0, 3, 2]);
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: PatternModifyLog = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.section, 2.5);
        assert_eq!(deserialized.modify, Some(vec![1, 0, 3, 2]));
    }

    // -- IRConnectionManager --

    #[test]
    fn ir_connection_manager_empty_name_returns_none() {
        assert!(IRConnectionManager::get_ir_connection_class("").is_none());
    }

    #[test]
    fn ir_connection_manager_nonempty_name_returns_some() {
        assert!(IRConnectionManager::get_ir_connection_class("LR2IR").is_some());
    }
}

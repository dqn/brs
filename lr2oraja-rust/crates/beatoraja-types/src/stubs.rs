// Phase 5+ stubs — types from downstream crates that beatoraja-core cannot import
// due to circular dependency constraints. These will be replaced if/when the
// dependency graph is restructured (e.g., extracting shared types into a common crate).

// ---------------------------------------------------------------------------
// beatoraja-play stubs
// ---------------------------------------------------------------------------

// GrooveGauge moved to beatoraja-types/src/groove_gauge.rs (Phase 15b)
pub use crate::groove_gauge::GrooveGauge;

/// Stub for beatoraja.play.JudgeAlgorithm
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JudgeAlgorithm {
    Combo,
    Duration,
    Lowest,
    Timing,
}

impl JudgeAlgorithm {
    pub fn name(&self) -> &str {
        match self {
            JudgeAlgorithm::Combo => "Combo",
            JudgeAlgorithm::Duration => "Duration",
            JudgeAlgorithm::Lowest => "Lowest",
            JudgeAlgorithm::Timing => "Timing",
        }
    }

    pub fn get_index(name: &str) -> i32 {
        match name {
            "Combo" => 0,
            "Duration" => 1,
            "Lowest" => 2,
            "Timing" => 3,
            _ => -1,
        }
    }

    pub fn values() -> &'static [JudgeAlgorithm] {
        &[
            JudgeAlgorithm::Combo,
            JudgeAlgorithm::Duration,
            JudgeAlgorithm::Lowest,
            JudgeAlgorithm::Timing,
        ]
    }
}

/// Stub for beatoraja.play.BMSPlayerRule
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BMSPlayerRule {
    LR2,
    Beatoraja,
}

// ---------------------------------------------------------------------------
// beatoraja-skin: SkinType moved to beatoraja-types/src/skin_type.rs
// ---------------------------------------------------------------------------

pub use crate::skin_type::SkinType;

// ---------------------------------------------------------------------------
// beatoraja-select stubs
// ---------------------------------------------------------------------------

/// Stub for beatoraja.select.BarSorter
pub struct BarSorter;

#[derive(Clone, Debug)]
pub struct BarSorterEntry {
    name: &'static str,
}

impl BarSorterEntry {
    pub fn name(&self) -> &str {
        self.name
    }
}

#[allow(dead_code)]
impl BarSorter {
    pub const DEFAULT_SORTER: &'static [BarSorterEntry] = &[
        BarSorterEntry { name: "TITLE" },
        BarSorterEntry { name: "CLEAR" },
        BarSorterEntry { name: "SCORE" },
        BarSorterEntry { name: "MISSCOUNT" },
        BarSorterEntry { name: "DATE" },
        BarSorterEntry { name: "LEVEL" },
    ];
}

// ---------------------------------------------------------------------------
// beatoraja-pattern stubs
// ---------------------------------------------------------------------------

pub mod scroll_speed_modifier {
    #[derive(Clone, Debug)]
    pub enum Mode {
        Off,
        Variable,
        Fixed,
    }

    impl Mode {
        pub fn values() -> &'static [Mode] {
            &[Mode::Off, Mode::Variable, Mode::Fixed]
        }
    }
}

pub mod long_note_modifier {
    #[derive(Clone, Debug)]
    pub enum Mode {
        Off,
        Add,
        Remove,
    }

    impl Mode {
        pub fn values() -> &'static [Mode] {
            &[Mode::Off, Mode::Add, Mode::Remove]
        }
    }
}

pub mod mine_note_modifier {
    #[derive(Clone, Debug)]
    pub enum Mode {
        Off,
        Remove,
    }

    impl Mode {
        pub fn values() -> &'static [Mode] {
            &[Mode::Off, Mode::Remove]
        }
    }
}

// ---------------------------------------------------------------------------
// beatoraja-ir stubs
// ---------------------------------------------------------------------------

/// Stub for beatoraja.ir.IRConnectionManager
pub struct IRConnectionManager;

#[allow(dead_code)]
impl IRConnectionManager {
    pub fn get_all_available_ir_connection_name() -> Vec<String> {
        vec![]
    }

    pub fn get_ir_connection_class(_name: &str) -> Option<()> {
        Some(())
    }
}

// ---------------------------------------------------------------------------
// beatoraja-input stubs (incompatible field layout with beatoraja-input crate)
// ---------------------------------------------------------------------------

/// Stub for beatoraja.input.BMSPlayerInputDevice.Type
pub mod bms_player_input_device {
    #[allow(non_camel_case_types)]
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum Type {
        BM_CONTROLLER,
        KEYBOARD,
        MIDI,
        MOUSE,
    }
}

/// Stub for beatoraja.input.KeyInputLog (pub fields; beatoraja-input uses private fields)
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct KeyInputLog {
    pub time: i64,
    pub keycode: i32,
    pub pressed: bool,
}

impl KeyInputLog {
    pub fn validate(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// beatoraja-pattern stubs (incompatible field layout with beatoraja-pattern crate)
// ---------------------------------------------------------------------------

/// Stub for beatoraja.pattern.PatternModifyLog (field layout differs from beatoraja-pattern)
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PatternModifyLog {
    pub old_lane: i32,
    pub new_lane: i32,
}

impl PatternModifyLog {
    pub fn validate(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// beatoraja-song stubs — SongData moved to beatoraja-types/src/song_data.rs
// ---------------------------------------------------------------------------

pub use crate::song_data::SongData;

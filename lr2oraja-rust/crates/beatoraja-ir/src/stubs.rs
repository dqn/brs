// Stubs for external dependencies not yet available in the beatoraja-ir crate

use std::collections::HashMap;

use bms_model::mode::Mode;

/// Stub for beatoraja.song.SongData
/// Full implementation is in the `beatoraja.song` package (not yet translated)
#[derive(Clone, Debug, Default)]
pub struct SongData {
    pub title: String,
    pub subtitle: String,
    pub genre: String,
    pub artist: String,
    pub subartist: String,
    pub md5: String,
    pub sha256: String,
    pub url: String,
    pub appendurl: String,
    pub level: i32,
    pub judge: i32,
    pub minbpm: i32,
    pub maxbpm: i32,
    pub notes: i32,
    pub has_undefined_long_note: bool,
    pub has_long_note: bool,
    pub has_charge_note: bool,
    pub has_hell_charge_note: bool,
    pub has_mine_note: bool,
    pub has_random_sequence: bool,
    pub bpmstop: bool,
    pub bms_model: Option<BMSModelStub>,
}

impl SongData {
    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_subtitle(&self) -> &str {
        &self.subtitle
    }

    pub fn get_genre(&self) -> &str {
        &self.genre
    }

    pub fn get_artist(&self) -> &str {
        &self.artist
    }

    pub fn get_subartist(&self) -> &str {
        &self.subartist
    }

    pub fn get_md5(&self) -> &str {
        &self.md5
    }

    pub fn get_sha256(&self) -> &str {
        &self.sha256
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_appendurl(&self) -> &str {
        &self.appendurl
    }

    pub fn get_level(&self) -> i32 {
        self.level
    }

    pub fn get_judge(&self) -> i32 {
        self.judge
    }

    pub fn get_minbpm(&self) -> i32 {
        self.minbpm
    }

    pub fn get_maxbpm(&self) -> i32 {
        self.maxbpm
    }

    pub fn get_notes(&self) -> i32 {
        self.notes
    }

    pub fn has_undefined_long_note(&self) -> bool {
        self.has_undefined_long_note
    }

    pub fn has_long_note(&self) -> bool {
        self.has_long_note
    }

    pub fn has_charge_note(&self) -> bool {
        self.has_charge_note
    }

    pub fn has_hell_charge_note(&self) -> bool {
        self.has_hell_charge_note
    }

    pub fn has_mine_note(&self) -> bool {
        self.has_mine_note
    }

    pub fn has_random_sequence(&self) -> bool {
        self.has_random_sequence
    }

    pub fn is_bpmstop(&self) -> bool {
        self.bpmstop
    }

    pub fn get_bms_model(&self) -> Option<&BMSModelStub> {
        self.bms_model.as_ref()
    }

    pub fn shrink(&mut self) {
        // Placeholder
    }
}

/// Stub for BMSModel (subset of fields needed by IRChartData)
#[derive(Clone, Debug, Default)]
pub struct BMSModelStub {
    pub total: f64,
    pub mode: Option<Mode>,
    pub lntype: i32,
    pub values: HashMap<String, String>,
}

impl BMSModelStub {
    pub fn get_total(&self) -> f64 {
        self.total
    }

    pub fn get_mode(&self) -> Option<&Mode> {
        self.mode.as_ref()
    }

    pub fn get_lntype(&self) -> i32 {
        self.lntype
    }

    pub fn get_values(&self) -> &HashMap<String, String> {
        &self.values
    }
}

/// Stub for MainController
pub struct MainController;

impl MainController {
    pub fn get_ir_status(&self) -> &[IRStatusStub] {
        &[]
    }

    pub fn get_player_config(&self) -> &PlayerConfigStub {
        todo!("MainController.get_player_config stub")
    }
}

/// Stub for IRStatus
pub struct IRStatusStub {
    pub connection: Box<dyn super::ir_connection::IRConnection>,
}

/// Stub for PlayerConfig (subset)
pub struct PlayerConfigStub {
    pub lnmode: i32,
}

impl PlayerConfigStub {
    pub fn get_lnmode(&self) -> i32 {
        self.lnmode
    }
}

/// Stub for MainState trait (subset needed by RankingData)
pub trait MainStateAccessor {
    fn get_main_controller(&self) -> &MainController;
    fn get_score_data_property(&self) -> &ScoreDataPropertyStub;
}

/// Stub for ScoreDataProperty (subset)
pub struct ScoreDataPropertyStub {
    pub score: Option<beatoraja_core::score_data::ScoreData>,
}

impl ScoreDataPropertyStub {
    pub fn get_score_data(&self) -> Option<&beatoraja_core::score_data::ScoreData> {
        self.score.as_ref()
    }
}

/// Stub for beatoraja.modmenu.ImGuiNotify
pub struct ImGuiNotify;

impl ImGuiNotify {
    pub fn error(msg: &str) {
        log::error!("ImGuiNotify: {}", msg);
    }

    pub fn warning(msg: &str) {
        log::warn!("ImGuiNotify: {}", msg);
    }
}

/// Stub for beatoraja.pattern.Random (enum for random option types)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Random {
    IDENTITY,
    MIRROR,
    RANDOM,
}

/// Stub for beatoraja.pattern.LR2Random
pub struct LR2Random {
    state: u32,
}

impl LR2Random {
    pub fn new(seed: i32) -> Self {
        // LR2-specific MT19937 seeding
        let _state = seed as u32;
        // Simple LCG-based stub; real implementation in beatoraja-pattern
        Self { state: seed as u32 }
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        // Simplified stub - real MT implementation is in beatoraja-pattern
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.state >> 16) as i32).abs() % bound
    }
}

/// Stub for BMSDecoder.convertHexString
pub fn convert_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

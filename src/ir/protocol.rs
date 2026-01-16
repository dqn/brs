use serde::{Deserialize, Serialize};

use crate::game::{ClearLamp, GaugeType, RandomOption};

/// IR server type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IrServerType {
    #[default]
    Lr2Ir,
    MochaIr,
    MinIr,
    Custom,
}

impl IrServerType {
    pub fn display_name(&self) -> &'static str {
        match self {
            IrServerType::Lr2Ir => "LR2IR",
            IrServerType::MochaIr => "Mocha-IR",
            IrServerType::MinIr => "MinIR",
            IrServerType::Custom => "Custom",
        }
    }

    pub fn default_url(&self) -> &'static str {
        match self {
            IrServerType::Lr2Ir => "https://www.dream-pro.info/~lavalse/LR2IR/2",
            IrServerType::MochaIr => "https://mocha-repository.info/ir",
            IrServerType::MinIr => "https://minir.cc/api",
            IrServerType::Custom => "",
        }
    }
}

/// Play option flags for IR submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayOptionFlags {
    pub random_option: RandomOption,
    pub gauge_type: GaugeType,
    pub auto_scratch: bool,
    pub legacy_note: bool,
    pub expand_judge: bool,
    pub battle: bool,
}

impl PlayOptionFlags {
    /// Check if any assist options were used
    pub fn has_assist(&self) -> bool {
        self.auto_scratch || self.legacy_note || self.expand_judge
    }

    /// Encode as LR2IR option format
    pub fn to_lr2ir_option(&self) -> u32 {
        let mut option = 0u32;

        // Random option (bits 0-2)
        option |= match self.random_option {
            RandomOption::Off => 0,
            RandomOption::Mirror => 1,
            RandomOption::Random => 2,
            RandomOption::RRandom => 2, // R-RANDOM treated as RANDOM for IR
            RandomOption::SRandom => 3,
            RandomOption::HRandom => 4,
        };

        // Gauge type (bits 4-6)
        option |= match self.gauge_type {
            GaugeType::AssistEasy => 0 << 4,
            GaugeType::Easy => 1 << 4,
            GaugeType::Normal => 2 << 4,
            GaugeType::Hard => 3 << 4,
            GaugeType::ExHard => 4 << 4,
            GaugeType::Hazard => 5 << 4,
        };

        // Assist options (bits 8-10)
        if self.auto_scratch {
            option |= 1 << 8;
        }
        if self.legacy_note {
            option |= 1 << 9;
        }
        if self.expand_judge {
            option |= 1 << 10;
        }

        // Battle option (bit 12)
        if self.battle {
            option |= 1 << 12;
        }

        option
    }
}

/// Score submission data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreSubmission {
    /// Player ID on the IR server
    pub player_id: String,
    /// Chart hash (SHA256 for internal use)
    pub chart_hash: String,
    /// Chart MD5 hash (LR2IR compatible)
    pub chart_md5: String,
    /// EX score (PGREAT * 2 + GREAT)
    pub ex_score: u32,
    /// Clear lamp
    pub clear_lamp: ClearLamp,
    /// Maximum combo achieved
    pub max_combo: u32,
    /// PGREAT count
    pub pgreat_count: u32,
    /// GREAT count
    pub great_count: u32,
    /// GOOD count
    pub good_count: u32,
    /// BAD count
    pub bad_count: u32,
    /// POOR count (including empty POORs)
    pub poor_count: u32,
    /// Total notes in the chart
    pub total_notes: u32,
    /// Play options
    pub play_option: PlayOptionFlags,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Client version string
    pub client_version: String,
    /// Score hash for validation
    pub score_hash: String,
}

/// Response from score submission
#[derive(Debug, Clone, Deserialize)]
pub struct SubmissionResponse {
    /// Whether submission succeeded
    pub success: bool,
    /// Player's rank on the leaderboard
    pub rank: Option<u32>,
    /// Total number of players on leaderboard
    pub total_players: Option<u32>,
    /// Server message
    pub message: Option<String>,
}

/// Single ranking entry
#[derive(Debug, Clone, Deserialize)]
pub struct RankingEntry {
    /// Rank position
    pub rank: u32,
    /// Player name
    pub player_name: String,
    /// EX score
    pub ex_score: u32,
    /// Clear lamp
    pub clear_lamp: ClearLamp,
    /// Play timestamp
    pub timestamp: Option<u64>,
}

/// Chart ranking data
#[derive(Debug, Clone, Deserialize)]
pub struct ChartRanking {
    /// Chart hash
    pub chart_hash: String,
    /// Ranking entries
    pub entries: Vec<RankingEntry>,
    /// Total number of players
    pub total_players: u32,
}

/// IR submission state for UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IrSubmitState {
    #[default]
    Idle,
    Submitting,
    Success,
    Failed,
    Disabled,
}

impl IrSubmitState {
    pub fn display_text(&self) -> &'static str {
        match self {
            IrSubmitState::Idle => "",
            IrSubmitState::Submitting => "IR: Submitting...",
            IrSubmitState::Success => "IR: Submitted!",
            IrSubmitState::Failed => "IR: Failed",
            IrSubmitState::Disabled => "IR: Disabled",
        }
    }
}

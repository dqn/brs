use serde::{Deserialize, Serialize};

use crate::stubs::{BMSPlayerRule, JudgeAlgorithm, bms_player_input_device};

/// Play options and configuration at the time of scoring.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PlayOption {
    pub random: i32,
    pub option: i32,
    pub seed: i64,
    pub assist: i32,
    pub gauge: i32,
    #[serde(rename = "deviceType")]
    pub device_type: Option<bms_player_input_device::Type>,
    #[serde(rename = "judgeAlgorithm")]
    pub judge_algorithm: Option<JudgeAlgorithm>,
    pub rule: Option<BMSPlayerRule>,
    pub skin: Option<String>,
}

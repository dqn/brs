use serde::{Deserialize, Serialize};

/// Early/late judge counts for each judge type (PG, GR, GD, BD, PR, MS).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct JudgeCounts {
    pub epg: i32,
    pub lpg: i32,
    pub egr: i32,
    pub lgr: i32,
    pub egd: i32,
    pub lgd: i32,
    pub ebd: i32,
    pub lbd: i32,
    pub epr: i32,
    pub lpr: i32,
    pub ems: i32,
    pub lms: i32,
}

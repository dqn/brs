use serde::{Deserialize, Serialize};

/// Timing statistics (average judge, duration, averages, standard deviation).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TimingStats {
    pub avgjudge: i64,
    #[serde(rename = "totalDuration")]
    pub total_duration: i64,
    pub avg: i64,
    #[serde(rename = "totalAvg")]
    pub total_avg: i64,
    pub stddev: i64,
}

impl Default for TimingStats {
    fn default() -> Self {
        Self {
            avgjudge: i64::MAX,
            total_duration: 0,
            avg: i64::MAX,
            total_avg: 0,
            stddev: i64::MAX,
        }
    }
}

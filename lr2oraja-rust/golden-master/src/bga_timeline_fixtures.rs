use serde::Deserialize;

/// Root structure for BGA timeline GM test fixtures.
#[derive(Debug, Deserialize)]
pub struct BgaTimelineFixture {
    pub test_cases: Vec<BgaTimelineTestCase>,
}

/// A single BGA timeline test case exported from Java.
#[derive(Debug, Deserialize)]
pub struct BgaTimelineTestCase {
    pub filename: String,
    /// Raw BGA events from timelines (bga_id/layer_id = -1 means no change for that layer).
    pub events: Vec<BgaEventFixture>,
    /// State snapshots at 1-second intervals from simulated timeline scan.
    pub snapshots: Vec<BgaSnapshotFixture>,
}

/// A BGA event entry from Java timeline (may contain both BGA and layer changes).
#[derive(Debug, Deserialize)]
pub struct BgaEventFixture {
    pub time_ms: i32,
    pub bga_id: i32,
    pub layer_id: i32,
}

/// BGA state snapshot at a specific time.
#[derive(Debug, Deserialize)]
pub struct BgaSnapshotFixture {
    pub time_ms: i32,
    pub current_bga: i32,
    pub current_layer: i32,
}

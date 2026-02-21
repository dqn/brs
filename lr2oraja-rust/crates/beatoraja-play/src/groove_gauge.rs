// GrooveGauge, Gauge, GaugeModifier moved to beatoraja-types (Phase 15b)
// Only the `create` factory function remains here (depends on BMSPlayerRule).
pub use beatoraja_types::groove_gauge::*;

use crate::bms_player_rule::BMSPlayerRule;
use crate::gauge_property::GaugeProperty;
use bms_model::bms_model::BMSModel;
use bms_model::mode::Mode;

/// Factory function for creating a GrooveGauge with automatic gauge property selection.
/// This depends on BMSPlayerRule which cannot be moved to beatoraja-types.
pub fn create_groove_gauge(
    model: &BMSModel,
    gauge_type: i32,
    grade: i32,
    gauge: Option<GaugeProperty>,
) -> Option<GrooveGauge> {
    let id = if grade > 0 {
        // Course gauge
        if gauge_type <= 2 {
            6
        } else if gauge_type == 3 {
            7
        } else {
            8
        }
    } else {
        gauge_type
    };
    if id >= 0 {
        let gauge = gauge.unwrap_or_else(|| {
            let mode = model.get_mode().cloned().unwrap_or(Mode::BEAT_7K);
            BMSPlayerRule::get_bms_player_rule(&mode).gauge
        });
        Some(GrooveGauge::new(model, id, &gauge))
    } else {
        None
    }
}

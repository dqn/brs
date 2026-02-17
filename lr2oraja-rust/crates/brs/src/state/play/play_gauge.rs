// Gauge management — death handling, auto-shift, and logging.

use tracing::info;

use bms_rule::gauge_property::GaugeType;
use bms_skin::property_id::TIMER_FAILED;

use crate::state::StateContext;

use super::{GAUGE_LOG_INTERVAL_US, GaugeAutoShift, PlayPhase, PlayState};

impl PlayState {
    /// Handle gauge death. Returns true if play continues, false if transitioned to Failed.
    pub(super) fn handle_gauge_death(&mut self, ctx: &mut StateContext) -> bool {
        match self.gauge_auto_shift {
            GaugeAutoShift::None => {
                self.phase = PlayPhase::Failed;
                ctx.timer.set_timer_on(TIMER_FAILED);
                self.key_beam_stop = true;
                info!("Play: gauge death -> Failed");
                false
            }
            GaugeAutoShift::Continue => true,
            GaugeAutoShift::SurvivalToGroove => {
                if let Some(gauge) = &mut self.gauge {
                    let active = gauge.active_type();
                    if active == GaugeType::Hard || active == GaugeType::ExHard {
                        gauge.set_active_type(GaugeType::Normal);
                        info!("Play: GAS survival->groove");
                    }
                }
                true
            }
            GaugeAutoShift::BestClear => {
                self.shift_to_best_clear_gauge();
                true
            }
            GaugeAutoShift::SelectToUnder => {
                self.shift_to_lower_gauge();
                true
            }
        }
    }

    /// Shift to the best gauge that's still alive.
    fn shift_to_best_clear_gauge(&mut self) {
        let gauge = match &mut self.gauge {
            Some(g) => g,
            None => return,
        };
        let bottom_idx = self.bottom_gauge as usize;
        let types = [
            GaugeType::ExHard,
            GaugeType::Hard,
            GaugeType::Normal,
            GaugeType::Easy,
            GaugeType::AssistEasy,
        ];
        for &gt in &types {
            if (gt as usize) < bottom_idx {
                continue;
            }
            if gauge.value_of(gt) > 0.0 {
                gauge.set_active_type(gt);
                info!("Play: GAS bestclear -> {:?}", gt);
                return;
            }
        }
    }

    /// Shift to one gauge type below current.
    fn shift_to_lower_gauge(&mut self) {
        let gauge = match &mut self.gauge {
            Some(g) => g,
            None => return,
        };
        let active = gauge.active_type();
        let lower = match active {
            GaugeType::ExHard => Some(GaugeType::Hard),
            GaugeType::Hard => Some(GaugeType::Normal),
            GaugeType::Normal => Some(GaugeType::Easy),
            GaugeType::Easy => Some(GaugeType::AssistEasy),
            _ => None,
        };
        if let Some(gt) = lower
            && (gt as usize) >= (self.bottom_gauge as usize)
        {
            gauge.set_active_type(gt);
            info!("Play: GAS select-to-under -> {:?}", gt);
        }
    }

    /// Record gauge values at 500ms intervals.
    pub(super) fn record_gauge_log(&mut self, ptime_us: i64) {
        while self.last_gauge_log_time_us + GAUGE_LOG_INTERVAL_US <= ptime_us {
            self.last_gauge_log_time_us += GAUGE_LOG_INTERVAL_US;
            if let Some(gauge) = &self.gauge {
                let values: Vec<f32> = GaugeType::ALL
                    .iter()
                    .map(|&gt| gauge.value_of(gt))
                    .collect();
                self.gauge_log.push(values);
            }
        }
    }
}

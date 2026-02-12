// Scroll speed modifier — normalizes or randomizes scroll speed.
//
// Ported from Java: ScrollSpeedModifier.java

use bms_model::BmsModel;

use crate::modifier::{AssistLevel, PatternModifier};

/// Mode for scroll speed modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollSpeedMode {
    /// Remove BPM changes and stop events, normalize to initial BPM
    Remove,
    /// Add random scroll speed changes every N sections
    Add,
}

/// Normalizes or randomizes scroll speed.
///
/// Remove mode: normalizes all BPM to initial, removes stop events.
/// Add mode: randomizes scroll speed per section (stub — requires scroll field).
///
/// Java: `ScrollSpeedModifier`
pub struct ScrollSpeedModifier {
    pub mode: ScrollSpeedMode,
    /// Section interval for scroll changes (Add mode)
    pub section: u32,
    /// Scroll rate variance (Add mode)
    pub rate: f64,
    /// Track assist level
    assist: AssistLevel,
}

impl ScrollSpeedModifier {
    pub fn new(mode: ScrollSpeedMode) -> Self {
        Self {
            mode,
            section: 4,
            rate: 0.5,
            assist: AssistLevel::None,
        }
    }

    pub fn with_section(mut self, section: u32) -> Self {
        self.section = section;
        self
    }

    pub fn with_rate(mut self, rate: f64) -> Self {
        self.rate = rate;
        self
    }
}

impl PatternModifier for ScrollSpeedModifier {
    fn modify(&mut self, model: &mut BmsModel) {
        match self.mode {
            ScrollSpeedMode::Remove => self.remove_scroll_changes(model),
            ScrollSpeedMode::Add => {
                // Add mode requires per-timeline scroll field which is not
                // present in the flat-note BmsModel. This is a stub.
                // The Java implementation modifies TimeLine.scroll which we
                // don't have in our model. No-op for now.
            }
        }
    }

    fn assist_level(&self) -> AssistLevel {
        self.assist
    }
}

impl ScrollSpeedModifier {
    fn remove_scroll_changes(&mut self, model: &mut BmsModel) {
        let initial_bpm = model.initial_bpm;

        // Check if there are any changes to remove
        let has_bpm_changes = model.bpm_changes.iter().any(|c| c.bpm != initial_bpm);
        let has_stops = !model.stop_events.is_empty();

        if has_bpm_changes || has_stops {
            self.assist = AssistLevel::LightAssist;
        }

        // Normalize all BPM changes to initial BPM
        for change in &mut model.bpm_changes {
            change.bpm = initial_bpm;
        }

        // Remove all stop events
        model.stop_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::{BpmChange, Note, PlayMode, StopEvent};

    fn make_model_with_bpm(bpm_changes: Vec<BpmChange>, stop_events: Vec<StopEvent>) -> BmsModel {
        BmsModel {
            mode: PlayMode::Beat7K,
            initial_bpm: 150.0,
            bpm_changes,
            stop_events,
            notes: vec![Note::normal(0, 1000, 1)],
            ..Default::default()
        }
    }

    #[test]
    fn test_remove_normalizes_bpm() {
        let bpm_changes = vec![
            BpmChange {
                time_us: 0,
                bpm: 150.0,
            },
            BpmChange {
                time_us: 1000000,
                bpm: 200.0,
            },
            BpmChange {
                time_us: 2000000,
                bpm: 100.0,
            },
        ];
        let mut model = make_model_with_bpm(bpm_changes, Vec::new());
        let mut modifier = ScrollSpeedModifier::new(ScrollSpeedMode::Remove);
        modifier.modify(&mut model);

        assert!(model.bpm_changes.iter().all(|c| c.bpm == 150.0));
        assert_eq!(modifier.assist_level(), AssistLevel::LightAssist);
    }

    #[test]
    fn test_remove_clears_stops() {
        let stop_events = vec![
            StopEvent {
                time_us: 500000,
                duration_ticks: 48,
                duration_us: 100000,
            },
            StopEvent {
                time_us: 1500000,
                duration_ticks: 96,
                duration_us: 200000,
            },
        ];
        let mut model = make_model_with_bpm(Vec::new(), stop_events);
        let mut modifier = ScrollSpeedModifier::new(ScrollSpeedMode::Remove);
        modifier.modify(&mut model);

        assert!(model.stop_events.is_empty());
        assert_eq!(modifier.assist_level(), AssistLevel::LightAssist);
    }

    #[test]
    fn test_remove_no_changes_no_assist() {
        let bpm_changes = vec![BpmChange {
            time_us: 0,
            bpm: 150.0, // same as initial
        }];
        let mut model = make_model_with_bpm(bpm_changes, Vec::new());
        let mut modifier = ScrollSpeedModifier::new(ScrollSpeedMode::Remove);
        modifier.modify(&mut model);

        assert_eq!(modifier.assist_level(), AssistLevel::None);
    }

    #[test]
    fn test_add_mode_noop() {
        let mut model = make_model_with_bpm(Vec::new(), Vec::new());
        let mut modifier = ScrollSpeedModifier::new(ScrollSpeedMode::Add);
        modifier.modify(&mut model);

        // Add mode is a stub, should not change anything
        assert_eq!(modifier.assist_level(), AssistLevel::None);
    }
}

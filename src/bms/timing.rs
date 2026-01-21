use std::cmp::Ordering;

use fraction::Fraction;

use super::TimingData;

/// Timing event for unified processing
#[derive(Debug, Clone)]
enum TimingEvent {
    BpmChange {
        measure: u32,
        position: Fraction,
        bpm: f64,
    },
    Stop {
        measure: u32,
        position: Fraction,
        duration_192: u32,
    },
}

impl TimingEvent {
    fn measure(&self) -> u32 {
        match self {
            Self::BpmChange { measure, .. } => *measure,
            Self::Stop { measure, .. } => *measure,
        }
    }

    fn position(&self) -> &Fraction {
        match self {
            Self::BpmChange { position, .. } => position,
            Self::Stop { position, .. } => position,
        }
    }

    /// Event ordering: BPM changes come before STOPs at the same position
    fn event_order(&self) -> u8 {
        match self {
            Self::BpmChange { .. } => 0,
            Self::Stop { .. } => 1,
        }
    }
}

/// Compare two Fractions safely.
///
/// This custom implementation is used instead of Fraction's built-in Ord because:
/// 1. It handles edge cases where numer()/denom() return None (NaN-like values)
/// 2. It explicitly checks for zero denominators to avoid division by zero panics
/// 3. It uses i128 cross-multiplication for precise integer comparison
fn compare_fractions(a: &Fraction, b: &Fraction) -> Ordering {
    let (a_numer, a_denom) = (a.numer(), a.denom());
    let (b_numer, b_denom) = (b.numer(), b.denom());

    match (a_numer, a_denom, b_numer, b_denom) {
        (Some(an), Some(ad), Some(bn), Some(bd)) if *ad != 0 && *bd != 0 => {
            // Cross multiply to avoid floating point: a/b vs c/d => a*d vs c*b
            let left = (*an as i128) * (*bd as i128);
            let right = (*bn as i128) * (*ad as i128);
            left.cmp(&right)
        }
        _ => Ordering::Equal,
    }
}

/// Check if a < b for Fractions
fn fraction_lt(a: &Fraction, b: &Fraction) -> bool {
    matches!(compare_fractions(a, b), Ordering::Less)
}

/// Convert Fraction to f64
fn fraction_to_f64(f: &Fraction) -> f64 {
    let (numer, denom) = (f.numer(), f.denom());
    match (numer, denom) {
        (Some(n), Some(d)) if *d != 0 => *n as f64 / *d as f64,
        _ => 0.0,
    }
}

/// Build sorted timing events from TimingData.
///
/// TODO: Performance optimization - consider caching the built events in TimingData
/// to avoid rebuilding on every `calculate_time_ms` call. This would require:
/// 1. Adding a cached_events field to TimingData (with skip_serializing)
/// 2. Using OnceCell or lazy initialization
/// 3. Invalidating cache if timing data changes
fn build_timing_events(timing: &TimingData) -> Vec<TimingEvent> {
    let mut events = Vec::with_capacity(timing.bpm_changes.len() + timing.stops.len());

    for change in &timing.bpm_changes {
        events.push(TimingEvent::BpmChange {
            measure: change.measure,
            position: change.position,
            bpm: change.bpm,
        });
    }

    for stop in &timing.stops {
        events.push(TimingEvent::Stop {
            measure: stop.measure,
            position: stop.position,
            duration_192: stop.duration_192,
        });
    }

    // Sort by (measure, position, event_order)
    events.sort_by(|a, b| match a.measure().cmp(&b.measure()) {
        Ordering::Equal => match compare_fractions(a.position(), b.position()) {
            Ordering::Equal => a.event_order().cmp(&b.event_order()),
            other => other,
        },
        other => other,
    });

    events
}

pub fn calculate_time_ms(measure: u32, position: Fraction, timing: &TimingData) -> f64 {
    let events = build_timing_events(timing);
    let zero = Fraction::new(0u32, 1u32);

    let mut time_ms = 0.0;
    let mut current_bpm = timing.initial_bpm;
    let mut current_measure = 0u32;
    let mut current_position = zero;

    // Process all events up to the target position
    for event in &events {
        let event_measure = event.measure();
        let event_position = event.position();

        // Skip events after our target
        if event_measure > measure
            || (event_measure == measure && fraction_lt(&position, event_position))
        {
            break;
        }

        // Advance time from current position to event position
        time_ms += calculate_interval(
            current_measure,
            &current_position,
            event_measure,
            event_position,
            current_bpm,
            timing,
        );

        // Apply the event
        match event {
            TimingEvent::BpmChange { bpm, .. } => {
                current_bpm = *bpm;
            }
            TimingEvent::Stop { duration_192, .. } => {
                let stop_beats = *duration_192 as f64 / 192.0 * 4.0;
                let stop_ms = stop_beats * (60000.0 / current_bpm);
                time_ms += stop_ms;
            }
        }

        current_measure = event_measure;
        current_position = *event_position;
    }

    // Add remaining time from last event to target position
    time_ms += calculate_interval(
        current_measure,
        &current_position,
        measure,
        &position,
        current_bpm,
        timing,
    );

    time_ms
}

/// Calculate time interval between two positions
fn calculate_interval(
    from_measure: u32,
    from_position: &Fraction,
    to_measure: u32,
    to_position: &Fraction,
    bpm: f64,
    timing: &TimingData,
) -> f64 {
    if from_measure == to_measure
        && compare_fractions(from_position, to_position) == Ordering::Equal
    {
        return 0.0;
    }

    let ms_per_beat = 60000.0 / bpm;
    let mut time_ms = 0.0;

    if from_measure == to_measure {
        // Same measure: just calculate the difference
        let measure_length = get_measure_length(from_measure, timing);
        let from_f64 = fraction_to_f64(from_position);
        let to_f64 = fraction_to_f64(to_position);
        let beats = 4.0 * measure_length * (to_f64 - from_f64);
        time_ms += beats * ms_per_beat;
    } else {
        // Different measures
        // 1. Remaining part of from_measure
        let from_length = get_measure_length(from_measure, timing);
        let from_f64 = fraction_to_f64(from_position);
        let remaining_beats = 4.0 * from_length * (1.0 - from_f64);
        time_ms += remaining_beats * ms_per_beat;

        // 2. Full measures in between
        for m in (from_measure + 1)..to_measure {
            let measure_length = get_measure_length(m, timing);
            let beats = 4.0 * measure_length;
            time_ms += beats * ms_per_beat;
        }

        // 3. Part of to_measure up to to_position
        let to_length = get_measure_length(to_measure, timing);
        let to_f64 = fraction_to_f64(to_position);
        let start_beats = 4.0 * to_length * to_f64;
        time_ms += start_beats * ms_per_beat;
    }

    time_ms
}

fn get_measure_length(measure: u32, timing: &TimingData) -> f64 {
    timing
        .measure_lengths
        .iter()
        .find(|m| m.measure == measure)
        .map(|m| m.length)
        .unwrap_or(1.0)
}

/// Public API for converting beats to milliseconds
#[allow(dead_code)]
pub fn beats_to_ms(beats: f64, bpm: f64) -> f64 {
    beats * (60000.0 / bpm)
}

/// Public API for converting milliseconds to beats
#[allow(dead_code)]
pub fn ms_to_beats(ms: f64, bpm: f64) -> f64 {
    ms / (60000.0 / bpm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bms::{BpmChange, MeasureLength, StopEvent};

    fn create_timing_data(
        initial_bpm: f64,
        bpm_changes: Vec<BpmChange>,
        stops: Vec<StopEvent>,
        measure_lengths: Vec<MeasureLength>,
    ) -> TimingData {
        TimingData {
            initial_bpm,
            bpm_changes,
            stops,
            measure_lengths,
        }
    }

    #[test]
    fn test_basic_timing() {
        let timing = create_timing_data(120.0, vec![], vec![], vec![]);

        // At 120 BPM, one beat = 500ms
        // One measure = 4 beats = 2000ms
        let time = calculate_time_ms(0, Fraction::new(0u32, 1u32), &timing);
        assert!((time - 0.0).abs() < 0.001);

        let time = calculate_time_ms(1, Fraction::new(0u32, 1u32), &timing);
        assert!((time - 2000.0).abs() < 0.001);

        let time = calculate_time_ms(0, Fraction::new(1u32, 2u32), &timing);
        assert!((time - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_bpm_change_at_position_zero() {
        let timing = create_timing_data(
            120.0,
            vec![BpmChange {
                measure: 1,
                position: Fraction::new(0u32, 1u32),
                bpm: 240.0,
            }],
            vec![],
            vec![],
        );

        // Measure 0 at 120 BPM = 2000ms
        let time = calculate_time_ms(1, Fraction::new(0u32, 1u32), &timing);
        assert!((time - 2000.0).abs() < 0.001);

        // Measure 1 at 240 BPM = 1000ms, total = 3000ms
        let time = calculate_time_ms(2, Fraction::new(0u32, 1u32), &timing);
        assert!((time - 3000.0).abs() < 0.001);
    }

    #[test]
    fn test_bpm_change_mid_measure() {
        let timing = create_timing_data(
            120.0,
            vec![BpmChange {
                measure: 0,
                position: Fraction::new(1u32, 2u32),
                bpm: 240.0,
            }],
            vec![],
            vec![],
        );

        // First half at 120 BPM: 2 beats = 1000ms
        let time = calculate_time_ms(0, Fraction::new(1u32, 2u32), &timing);
        assert!((time - 1000.0).abs() < 0.001);

        // Second half at 240 BPM: 2 beats = 500ms
        let time = calculate_time_ms(1, Fraction::new(0u32, 1u32), &timing);
        assert!((time - 1500.0).abs() < 0.001);
    }

    #[test]
    fn test_stop_event() {
        let timing = create_timing_data(
            120.0,
            vec![],
            vec![StopEvent {
                measure: 0,
                position: Fraction::new(1u32, 2u32),
                duration_192: 48, // 1 beat stop
            }],
            vec![],
        );

        // At 120 BPM: 1 beat = 500ms stop
        // Position 1/2 = 1000ms + 500ms stop = 1500ms total before position 3/4
        let time = calculate_time_ms(0, Fraction::new(3u32, 4u32), &timing);
        // 1/2 measure = 1000ms, stop = 500ms, 1/4 measure = 500ms
        assert!((time - 2000.0).abs() < 0.001);
    }

    #[test]
    fn test_multiple_bpm_changes_same_measure() {
        let timing = create_timing_data(
            120.0,
            vec![
                BpmChange {
                    measure: 0,
                    position: Fraction::new(1u32, 4u32),
                    bpm: 180.0,
                },
                BpmChange {
                    measure: 0,
                    position: Fraction::new(1u32, 2u32),
                    bpm: 240.0,
                },
            ],
            vec![],
            vec![],
        );

        // 0 to 1/4 at 120 BPM: 1 beat = 500ms
        // 1/4 to 1/2 at 180 BPM: 1 beat = 333.33ms
        // 1/2 to 1 at 240 BPM: 2 beats = 500ms
        let time = calculate_time_ms(1, Fraction::new(0u32, 1u32), &timing);
        let expected = 500.0 + (60000.0 / 180.0) + 500.0;
        assert!((time - expected).abs() < 0.01);
    }

    #[test]
    fn test_measure_length() {
        let timing = create_timing_data(
            120.0,
            vec![],
            vec![],
            vec![MeasureLength {
                measure: 0,
                length: 0.5,
            }],
        );

        // Measure 0 is half length: 2 beats = 1000ms
        let time = calculate_time_ms(1, Fraction::new(0u32, 1u32), &timing);
        assert!((time - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_fraction_comparison() {
        let a = Fraction::new(1u32, 4u32);
        let b = Fraction::new(2u32, 8u32);
        assert_eq!(compare_fractions(&a, &b), Ordering::Equal);

        let c = Fraction::new(1u32, 3u32);
        assert_eq!(compare_fractions(&a, &c), Ordering::Less);
        assert_eq!(compare_fractions(&c, &a), Ordering::Greater);
    }

    #[test]
    fn test_bpm_change_and_stop_at_same_position() {
        // BPM change should happen before STOP at the same position
        let timing = create_timing_data(
            120.0,
            vec![BpmChange {
                measure: 0,
                position: Fraction::new(1u32, 2u32),
                bpm: 240.0,
            }],
            vec![StopEvent {
                measure: 0,
                position: Fraction::new(1u32, 2u32),
                duration_192: 48, // 1 beat stop at 240 BPM = 250ms
            }],
            vec![],
        );

        // First half at 120 BPM: 2 beats = 1000ms
        // STOP at 240 BPM: 1 beat = 250ms
        // Second half at 240 BPM: 2 beats = 500ms
        let time = calculate_time_ms(1, Fraction::new(0u32, 1u32), &timing);
        assert!((time - 1750.0).abs() < 0.01);
    }
}

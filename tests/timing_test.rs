use bms_player::bms::{BpmChange, MeasureLength, StopEvent, TimingData, calculate_time_ms};
use fraction::Fraction;

fn default_timing() -> TimingData {
    TimingData {
        initial_bpm: 120.0,
        bpm_changes: vec![],
        stops: vec![],
        measure_lengths: vec![],
    }
}

#[test]
fn test_simple_timing() {
    let timing = default_timing();

    // At 120 BPM, one beat = 500ms, one measure = 2000ms
    let ms = calculate_time_ms(0, Fraction::new(0u64, 1u64), &timing);
    assert!(
        (ms - 0.0).abs() < 0.001,
        "Measure 0, position 0 should be 0ms"
    );

    let ms = calculate_time_ms(0, Fraction::new(1u64, 2u64), &timing);
    assert!(
        (ms - 1000.0).abs() < 0.001,
        "Measure 0, position 0.5 should be 1000ms"
    );

    let ms = calculate_time_ms(1, Fraction::new(0u64, 1u64), &timing);
    assert!(
        (ms - 2000.0).abs() < 0.001,
        "Measure 1, position 0 should be 2000ms"
    );
}

#[test]
fn test_bpm_change() {
    let timing = TimingData {
        initial_bpm: 120.0,
        bpm_changes: vec![BpmChange {
            measure: 1,
            position: Fraction::new(0u64, 1u64),
            bpm: 60.0,
        }],
        stops: vec![],
        measure_lengths: vec![],
    };

    // Before BPM change
    let ms = calculate_time_ms(0, Fraction::new(1u64, 2u64), &timing);
    assert!(
        (ms - 1000.0).abs() < 0.001,
        "Before BPM change should be unaffected"
    );

    // After BPM change (60 BPM = 4000ms per measure)
    let ms = calculate_time_ms(2, Fraction::new(0u64, 1u64), &timing);
    // Measure 0-1: 2000ms at 120 BPM
    // Measure 1-2: 4000ms at 60 BPM
    assert!(
        (ms - 6000.0).abs() < 0.001,
        "After BPM change should use new BPM"
    );
}

#[test]
fn test_measure_length() {
    let timing = TimingData {
        initial_bpm: 120.0,
        bpm_changes: vec![],
        stops: vec![],
        measure_lengths: vec![MeasureLength {
            measure: 1,
            length: 0.5, // 2/4 time
        }],
    };

    // Measure 0: normal (2000ms)
    // Measure 1: half length (1000ms)
    let ms = calculate_time_ms(2, Fraction::new(0u64, 1u64), &timing);
    assert!(
        (ms - 3000.0).abs() < 0.001,
        "Short measure should reduce time"
    );
}

#[test]
fn test_stop_event() {
    let timing = TimingData {
        initial_bpm: 120.0,
        bpm_changes: vec![],
        stops: vec![StopEvent {
            measure: 0,
            position: Fraction::new(1u64, 2u64),
            duration_192: 192, // 1 whole note = 2000ms at 120 BPM
        }],
        measure_lengths: vec![],
    };

    // After the stop at measure 0, position 0.5
    let ms = calculate_time_ms(1, Fraction::new(0u64, 1u64), &timing);
    // Normal: 2000ms + stop: 2000ms = 4000ms
    assert!(
        (ms - 4000.0).abs() < 0.001,
        "Stop should add duration: expected 4000, got {}",
        ms
    );
}

use fraction::Fraction;

use super::TimingData;

pub fn calculate_time_ms(measure: u32, position: Fraction, timing: &TimingData) -> f64 {
    let mut time_ms = 0.0;
    let mut current_bpm = timing.initial_bpm;

    for m in 0..measure {
        // Apply BPM changes at position 0 before calculating measure duration
        for change in &timing.bpm_changes {
            if change.measure == m && fraction_to_f64(&change.position) == 0.0 {
                current_bpm = change.bpm;
            }
        }

        let measure_length = get_measure_length(m, timing);
        let beats = 4.0 * measure_length;
        let ms_per_beat = 60000.0 / current_bpm;
        time_ms += beats * ms_per_beat;

        // Apply BPM changes at non-zero positions (for subsequent measures)
        for change in &timing.bpm_changes {
            if change.measure == m && fraction_to_f64(&change.position) > 0.0 {
                current_bpm = change.bpm;
            }
        }

        for stop in &timing.stops {
            if stop.measure == m {
                let stop_beats = stop.duration_192 as f64 / 192.0 * 4.0;
                let stop_ms = stop_beats * (60000.0 / current_bpm);
                time_ms += stop_ms;
            }
        }
    }

    // Apply BPM changes at position 0 of target measure
    for change in &timing.bpm_changes {
        if change.measure == measure && fraction_to_f64(&change.position) == 0.0 {
            current_bpm = change.bpm;
        }
    }

    let measure_length = get_measure_length(measure, timing);
    let pos_f64 = fraction_to_f64(&position);
    let beats = 4.0 * measure_length * pos_f64;
    let ms_per_beat = 60000.0 / current_bpm;
    time_ms += beats * ms_per_beat;

    for change in &timing.bpm_changes {
        if change.measure == measure {
            let change_pos = fraction_to_f64(&change.position);
            if change_pos > 0.0 && change_pos <= pos_f64 {
                let delta_pos = pos_f64 - change_pos;
                let old_ms = delta_pos * 4.0 * measure_length * (60000.0 / current_bpm);
                current_bpm = change.bpm;
                let new_ms = delta_pos * 4.0 * measure_length * (60000.0 / current_bpm);
                time_ms = time_ms - old_ms + new_ms;
            }
        }
    }

    for stop in &timing.stops {
        if stop.measure == measure {
            let stop_pos = fraction_to_f64(&stop.position);
            if stop_pos <= pos_f64 {
                let stop_beats = stop.duration_192 as f64 / 192.0 * 4.0;
                let stop_ms = stop_beats * (60000.0 / current_bpm);
                time_ms += stop_ms;
            }
        }
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

fn fraction_to_f64(f: &Fraction) -> f64 {
    let (numer, denom) = (f.numer(), f.denom());
    match (numer, denom) {
        (Some(n), Some(d)) if *d != 0 => *n as f64 / *d as f64,
        _ => 0.0,
    }
}

// Public API for converting beats to milliseconds
#[allow(dead_code)]
pub fn beats_to_ms(beats: f64, bpm: f64) -> f64 {
    beats * (60000.0 / bpm)
}

// Public API for converting milliseconds to beats
#[allow(dead_code)]
pub fn ms_to_beats(ms: f64, bpm: f64) -> f64 {
    ms / (60000.0 / bpm)
}

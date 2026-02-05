/// Timing engine for converting BMS object positions to microseconds.
///
/// BMS uses a measure-fraction system (e.g., measure 3, position 0.5 = halfway through measure 3).
/// This engine converts those positions to absolute microseconds, considering BPM changes and stops.
///
/// Corresponds to the timing logic in beatoraja's BMSModel/TimeLine.
/// A position within the BMS chart as (measure, fraction within measure).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ObjTime {
    /// Measure number (0-indexed).
    pub measure: u32,
    /// Position within the measure (0.0..1.0).
    pub fraction: f64,
}

impl ObjTime {
    pub fn new(measure: u32, fraction: f64) -> Self {
        Self { measure, fraction }
    }

    /// Convert to a total "beat position" assuming 4/4 time.
    /// Each measure = 4 beats by default, but BMS allows variable measure lengths.
    pub fn to_total_position(&self, measure_lengths: &[f64]) -> f64 {
        let mut total = 0.0;
        for m in 0..self.measure {
            let len = measure_lengths.get(m as usize).copied().unwrap_or(1.0);
            total += len * 4.0; // 4 beats per standard measure
        }
        let current_len = measure_lengths
            .get(self.measure as usize)
            .copied()
            .unwrap_or(1.0);
        total += self.fraction * current_len * 4.0;
        total
    }
}

/// BPM change at a specific beat position.
#[derive(Debug, Clone, PartialEq)]
pub struct BpmEvent {
    /// Beat position (total beats from start).
    pub beat_position: f64,
    /// New BPM.
    pub bpm: f64,
}

/// Stop event at a specific beat position.
#[derive(Debug, Clone, PartialEq)]
pub struct StopSpec {
    /// Beat position where the stop occurs.
    pub beat_position: f64,
    /// Stop duration in beats (192nds in BMS: value / 192.0 * 4.0).
    pub duration_beats: f64,
}

/// Engine for converting beat positions to microseconds.
#[derive(Debug, Clone)]
pub struct TimingEngine {
    /// Initial BPM.
    initial_bpm: f64,
    /// BPM changes sorted by beat position.
    bpm_changes: Vec<BpmEvent>,
    /// Stops sorted by beat position.
    stops: Vec<StopSpec>,
    /// Precomputed timing points for fast lookup.
    timing_points: Vec<TimingPoint>,
}

/// Precomputed timing point.
#[derive(Debug, Clone)]
struct TimingPoint {
    /// Beat position.
    beat_position: f64,
    /// Time in microseconds at this beat position.
    time_us: i64,
    /// BPM active from this point.
    bpm: f64,
}

impl TimingEngine {
    /// Create a new timing engine from BPM changes and stop events.
    ///
    /// `bpm_changes` and `stops` must be sorted by beat_position.
    pub fn new(initial_bpm: f64, bpm_changes: Vec<BpmEvent>, stops: Vec<StopSpec>) -> Self {
        let mut engine = Self {
            initial_bpm,
            bpm_changes,
            stops,
            timing_points: Vec::new(),
        };
        engine.build_timing_points();
        engine
    }

    /// Build precomputed timing points from BPM changes and stops.
    fn build_timing_points(&mut self) {
        // Collect all events (BPM changes and stops) and sort them
        #[derive(Debug, Clone)]
        enum Event {
            BpmChange(f64, f64), // (beat_pos, new_bpm)
            Stop(f64, f64),      // (beat_pos, duration_beats)
        }

        let mut events: Vec<Event> = Vec::new();
        for bc in &self.bpm_changes {
            events.push(Event::BpmChange(bc.beat_position, bc.bpm));
        }
        for s in &self.stops {
            events.push(Event::Stop(s.beat_position, s.duration_beats));
        }

        // Sort by beat position, BPM changes before stops at the same position
        events.sort_by(|a, b| {
            let pos_a = match a {
                Event::BpmChange(p, _) | Event::Stop(p, _) => *p,
            };
            let pos_b = match b {
                Event::BpmChange(p, _) | Event::Stop(p, _) => *p,
            };
            pos_a.partial_cmp(&pos_b).unwrap().then_with(|| {
                // BPM changes before stops at the same position
                let order_a = match a {
                    Event::BpmChange(_, _) => 0,
                    Event::Stop(_, _) => 1,
                };
                let order_b = match b {
                    Event::BpmChange(_, _) => 0,
                    Event::Stop(_, _) => 1,
                };
                order_a.cmp(&order_b)
            })
        });

        self.timing_points.clear();
        let mut current_bpm = self.initial_bpm;
        let mut current_beat = 0.0_f64;
        let mut current_time_us = 0_i64;

        // Initial timing point
        self.timing_points.push(TimingPoint {
            beat_position: 0.0,
            time_us: 0,
            bpm: current_bpm,
        });

        for event in &events {
            match event {
                Event::BpmChange(beat_pos, new_bpm) => {
                    let delta_beats = beat_pos - current_beat;
                    if delta_beats > 0.0 {
                        let delta_us = beats_to_us(delta_beats, current_bpm);
                        current_time_us += delta_us;
                        current_beat = *beat_pos;
                    }
                    current_bpm = *new_bpm;
                    self.timing_points.push(TimingPoint {
                        beat_position: current_beat,
                        time_us: current_time_us,
                        bpm: current_bpm,
                    });
                }
                Event::Stop(beat_pos, duration_beats) => {
                    let delta_beats = beat_pos - current_beat;
                    if delta_beats > 0.0 {
                        let delta_us = beats_to_us(delta_beats, current_bpm);
                        current_time_us += delta_us;
                        current_beat = *beat_pos;
                    }
                    // Add stop time
                    let stop_us = beats_to_us(*duration_beats, current_bpm);
                    current_time_us += stop_us;
                    // Push a timing point after the stop (same beat position, later time)
                    self.timing_points.push(TimingPoint {
                        beat_position: current_beat,
                        time_us: current_time_us,
                        bpm: current_bpm,
                    });
                }
            }
        }
    }

    /// Convert a beat position to microseconds.
    pub fn beat_to_us(&self, beat_position: f64) -> i64 {
        // Find the last timing point at or before this beat position
        let mut best = &self.timing_points[0];
        for tp in &self.timing_points {
            if tp.beat_position <= beat_position {
                best = tp;
            } else {
                break;
            }
        }
        let delta_beats = beat_position - best.beat_position;
        best.time_us + beats_to_us(delta_beats, best.bpm)
    }

    /// Convert an ObjTime to microseconds.
    pub fn obj_time_to_us(&self, obj_time: ObjTime, measure_lengths: &[f64]) -> i64 {
        let beat_position = obj_time.to_total_position(measure_lengths);
        self.beat_to_us(beat_position)
    }

    /// Get the BPM at a given beat position.
    pub fn bpm_at(&self, beat_position: f64) -> f64 {
        let mut bpm = self.initial_bpm;
        for tp in &self.timing_points {
            if tp.beat_position <= beat_position {
                bpm = tp.bpm;
            } else {
                break;
            }
        }
        bpm
    }

    pub fn initial_bpm(&self) -> f64 {
        self.initial_bpm
    }
}

/// Convert beats to microseconds at a given BPM.
/// 1 beat = 60 / BPM seconds = 60_000_000 / BPM microseconds.
fn beats_to_us(beats: f64, bpm: f64) -> i64 {
    if bpm <= 0.0 {
        return 0;
    }
    (beats * 60_000_000.0 / bpm) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_bpm() {
        let engine = TimingEngine::new(120.0, vec![], vec![]);
        // At 120 BPM, 1 beat = 500ms = 500000us
        assert_eq!(engine.beat_to_us(0.0), 0);
        assert_eq!(engine.beat_to_us(1.0), 500_000);
        assert_eq!(engine.beat_to_us(4.0), 2_000_000);
    }

    #[test]
    fn bpm_change() {
        let engine = TimingEngine::new(
            120.0,
            vec![BpmEvent {
                beat_position: 4.0,
                bpm: 240.0,
            }],
            vec![],
        );
        // First 4 beats at 120 BPM = 2,000,000us
        assert_eq!(engine.beat_to_us(4.0), 2_000_000);
        // After BPM change to 240: 1 beat = 250ms = 250000us
        assert_eq!(engine.beat_to_us(5.0), 2_250_000);
        assert_eq!(engine.beat_to_us(8.0), 3_000_000);
    }

    #[test]
    fn bpm_deceleration() {
        let engine = TimingEngine::new(
            240.0,
            vec![BpmEvent {
                beat_position: 4.0,
                bpm: 120.0,
            }],
            vec![],
        );
        // First 4 beats at 240 BPM: 4 * 250000 = 1,000,000
        assert_eq!(engine.beat_to_us(4.0), 1_000_000);
        // Next beat at 120 BPM: 500000
        assert_eq!(engine.beat_to_us(5.0), 1_500_000);
    }

    #[test]
    fn stop_event() {
        let engine = TimingEngine::new(
            120.0,
            vec![],
            vec![StopSpec {
                beat_position: 4.0,
                duration_beats: 2.0,
            }],
        );
        // At beat 4.0, stop for 2 beats (at 120 BPM = 1,000,000us)
        // Beat 0-4: 2,000,000us
        // Stop: 1,000,000us
        // Beat 4+: continues from 3,000,000us
        assert_eq!(engine.beat_to_us(3.0), 1_500_000);
        assert_eq!(engine.beat_to_us(4.0), 3_000_000);
        assert_eq!(engine.beat_to_us(5.0), 3_500_000);
    }

    #[test]
    fn bpm_change_and_stop() {
        let engine = TimingEngine::new(
            120.0,
            vec![BpmEvent {
                beat_position: 4.0,
                bpm: 240.0,
            }],
            vec![StopSpec {
                beat_position: 4.0,
                duration_beats: 1.0,
            }],
        );
        // Beat 0-4 at 120 BPM: 2,000,000us
        // BPM changes to 240 at beat 4
        // Stop for 1 beat at 240 BPM = 250,000us
        // Beat 4 time = 2,000,000 + 250,000 = 2,250,000
        assert_eq!(engine.beat_to_us(4.0), 2_250_000);
        // Beat 5 at 240 BPM: 2,250,000 + 250,000 = 2,500,000
        assert_eq!(engine.beat_to_us(5.0), 2_500_000);
    }

    #[test]
    fn obj_time_conversion() {
        let engine = TimingEngine::new(120.0, vec![], vec![]);
        let measure_lengths = vec![1.0; 10]; // all standard measures

        // Measure 0, fraction 0.0 = beat 0
        assert_eq!(
            engine.obj_time_to_us(ObjTime::new(0, 0.0), &measure_lengths),
            0
        );

        // Measure 1, fraction 0.0 = beat 4
        assert_eq!(
            engine.obj_time_to_us(ObjTime::new(1, 0.0), &measure_lengths),
            2_000_000
        );

        // Measure 0, fraction 0.5 = beat 2
        assert_eq!(
            engine.obj_time_to_us(ObjTime::new(0, 0.5), &measure_lengths),
            1_000_000
        );
    }

    #[test]
    fn variable_measure_length() {
        let engine = TimingEngine::new(120.0, vec![], vec![]);
        let measure_lengths = vec![1.0, 0.5, 1.0]; // measure 1 is half length

        // Measure 0 = 4 beats, measure 1 = 2 beats
        // Start of measure 2 = beat 6
        let us = engine.obj_time_to_us(ObjTime::new(2, 0.0), &measure_lengths);
        assert_eq!(us, 3_000_000); // 6 beats at 120 BPM
    }

    #[test]
    fn bpm_at_position() {
        let engine = TimingEngine::new(
            120.0,
            vec![
                BpmEvent {
                    beat_position: 4.0,
                    bpm: 180.0,
                },
                BpmEvent {
                    beat_position: 8.0,
                    bpm: 90.0,
                },
            ],
            vec![],
        );
        assert_eq!(engine.bpm_at(0.0), 120.0);
        assert_eq!(engine.bpm_at(3.0), 120.0);
        assert_eq!(engine.bpm_at(4.0), 180.0);
        assert_eq!(engine.bpm_at(7.0), 180.0);
        assert_eq!(engine.bpm_at(8.0), 90.0);
    }

    #[test]
    fn monotonicity() {
        // Time should always increase (or stay same at stops) as beat position increases
        let engine = TimingEngine::new(
            120.0,
            vec![
                BpmEvent {
                    beat_position: 4.0,
                    bpm: 240.0,
                },
                BpmEvent {
                    beat_position: 8.0,
                    bpm: 60.0,
                },
            ],
            vec![StopSpec {
                beat_position: 6.0,
                duration_beats: 1.0,
            }],
        );

        let mut prev_us = i64::MIN;
        for i in 0..100 {
            let beat = i as f64 * 0.1;
            let us = engine.beat_to_us(beat);
            assert!(us >= prev_us, "time should be monotonic at beat {beat}");
            prev_us = us;
        }
    }
}

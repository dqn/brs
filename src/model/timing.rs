use bms_rs::bms::prelude::*;
use num_traits::ToPrimitive;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

#[derive(Debug, Clone)]
pub struct TimingEngine {
    initial_bpm: f64,
    measure_lengths: Vec<f64>,
    breakpoints: Vec<ObjTime>,
    time_at: Vec<f64>,
    time_after: Vec<f64>,
    bpm_after: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct BpmEvent {
    bpm: f64,
    priority: u8,
}

impl TimingEngine {
    pub fn new(bms: &Bms) -> Self {
        let initial_bpm = bms
            .bpm
            .bpm
            .as_ref()
            .and_then(|bpm| bpm.to_f64())
            .unwrap_or(120.0);
        Self::from_bms_with_initial(bms, initial_bpm)
    }

    pub fn initial_bpm(&self) -> f64 {
        self.initial_bpm
    }

    pub fn objtime_to_ms(&self, time: ObjTime) -> f64 {
        if self.breakpoints.is_empty() {
            return 0.0;
        }
        match self.breakpoints.binary_search(&time) {
            Ok(index) => self.time_at[index],
            Err(pos) => {
                if pos == 0 {
                    return 0.0;
                }
                let index = pos - 1;
                let base = self.breakpoints[index];
                let base_time = self.time_after[index];
                let bpm = self.bpm_after[index];
                let delta = Self::duration_between(base, time, bpm, &self.measure_lengths);
                base_time + delta
            }
        }
    }

    pub fn bpm_at(&self, time: ObjTime) -> f64 {
        if self.breakpoints.is_empty() {
            return self.initial_bpm;
        }
        match self.breakpoints.binary_search(&time) {
            Ok(index) => self.bpm_after[index],
            Err(pos) => {
                if pos == 0 {
                    self.initial_bpm
                } else {
                    self.bpm_after[pos - 1]
                }
            }
        }
    }

    fn from_bms_with_initial(bms: &Bms, initial_bpm: f64) -> Self {
        let max_measure = Self::max_measure(bms);
        let measure_lengths = Self::build_measure_lengths(bms, max_measure);
        let bpm_events = Self::collect_bpm_events(bms);
        let stop_events = Self::collect_stop_events(bms);
        let stp_events = Self::collect_stp_events(bms);
        let breakpoints =
            Self::collect_breakpoints(max_measure, &bpm_events, &stop_events, &stp_events);

        let mut time_at = Vec::with_capacity(breakpoints.len());
        let mut time_after = Vec::with_capacity(breakpoints.len());
        let mut bpm_after = Vec::with_capacity(breakpoints.len());

        let mut current_time = 0.0;
        let mut current_bpm = initial_bpm;
        let mut prev = *breakpoints.first().unwrap_or(&ObjTime::new(
            0,
            0,
            NonZeroU64::new(1).expect("1 should be a valid NonZeroU64"),
        ));

        for (index, point) in breakpoints.iter().enumerate() {
            if index == 0 {
                time_at.push(current_time);
            } else {
                current_time += Self::duration_between(prev, *point, current_bpm, &measure_lengths);
                time_at.push(current_time);
            }

            if let Some(events) = bpm_events.get(point) {
                for event in events {
                    current_bpm = event.bpm;
                }
            }

            let mut after = current_time;
            if let Some(units) = stop_events.get(point) {
                after += Self::stop_units_to_ms(*units, current_bpm);
            }
            if let Some(ms) = stp_events.get(point) {
                after += *ms;
            }

            time_after.push(after);
            bpm_after.push(current_bpm);
            current_time = after;
            prev = *point;
        }

        Self {
            initial_bpm,
            measure_lengths,
            breakpoints,
            time_at,
            time_after,
            bpm_after,
        }
    }

    fn max_measure(bms: &Bms) -> u64 {
        let mut max_measure = 0;
        let mut consider = |value: Option<u64>| {
            if let Some(value) = value {
                max_measure = max_measure.max(value);
            }
        };

        consider(bms.notes().all_notes().map(|n| n.offset.track().0).max());
        consider(bms.bpm.bpm_changes.keys().map(|t| t.track().0).max());
        consider(bms.bpm.bpm_changes_u8.keys().map(|t| t.track().0).max());
        consider(bms.stop.stops.keys().map(|t| t.track().0).max());
        consider(bms.stop.stp_events.keys().map(|t| t.track().0).max());
        consider(
            bms.section_len
                .section_len_changes
                .keys()
                .map(|t| t.0)
                .max(),
        );

        max_measure
    }

    fn build_measure_lengths(bms: &Bms, max_measure: u64) -> Vec<f64> {
        let mut lengths = vec![1.0; (max_measure + 1) as usize];
        for (track, change) in &bms.section_len.section_len_changes {
            let len = change.length.to_f64().unwrap_or(1.0);
            if len > 0.0 {
                lengths[track.0 as usize] = len;
            }
        }
        lengths
    }

    fn collect_bpm_events(bms: &Bms) -> BTreeMap<ObjTime, Vec<BpmEvent>> {
        let mut events: BTreeMap<ObjTime, Vec<BpmEvent>> = BTreeMap::new();
        for change in bms.bpm.bpm_changes.values() {
            if let Some(bpm) = change.bpm.to_f64() {
                events
                    .entry(change.time)
                    .or_default()
                    .push(BpmEvent { bpm, priority: 1 });
            }
        }
        for (time, bpm) in &bms.bpm.bpm_changes_u8 {
            events.entry(*time).or_default().push(BpmEvent {
                bpm: *bpm as f64,
                priority: 0,
            });
        }

        for list in events.values_mut() {
            list.sort_by_key(|event| event.priority);
        }

        events
    }

    fn collect_stop_events(bms: &Bms) -> BTreeMap<ObjTime, f64> {
        let mut events: BTreeMap<ObjTime, f64> = BTreeMap::new();
        for stop in bms.stop.stops.values() {
            let units = stop.duration.to_f64().unwrap_or(0.0);
            if units > 0.0 {
                *events.entry(stop.time).or_insert(0.0) += units;
            }
        }
        events
    }

    fn collect_stp_events(bms: &Bms) -> BTreeMap<ObjTime, f64> {
        let mut events: BTreeMap<ObjTime, f64> = BTreeMap::new();
        for event in bms.stop.stp_events.values() {
            let ms = event.duration.as_millis() as f64;
            if ms > 0.0 {
                *events.entry(event.time).or_insert(0.0) += ms;
            }
        }
        events
    }

    fn collect_breakpoints(
        max_measure: u64,
        bpm_events: &BTreeMap<ObjTime, Vec<BpmEvent>>,
        stop_events: &BTreeMap<ObjTime, f64>,
        stp_events: &BTreeMap<ObjTime, f64>,
    ) -> Vec<ObjTime> {
        let mut points = BTreeSet::new();
        for measure in 0..=max_measure {
            points.insert(ObjTime::new(
                measure,
                0,
                NonZeroU64::new(1).expect("1 should be a valid NonZeroU64"),
            ));
        }

        points.extend(bpm_events.keys().copied());
        points.extend(stop_events.keys().copied());
        points.extend(stp_events.keys().copied());

        points.into_iter().collect()
    }

    fn duration_between(from: ObjTime, to: ObjTime, bpm: f64, measure_lengths: &[f64]) -> f64 {
        if bpm <= 0.0 || from == to {
            return 0.0;
        }
        let from_track = from.track().0;
        let to_track = to.track().0;
        if from_track == to_track {
            let length = Self::measure_length(from_track, measure_lengths);
            let delta = Self::pos_in_measure(to) - Self::pos_in_measure(from);
            return Self::calc_duration(delta, length, bpm);
        }

        let mut total = 0.0;
        let from_len = Self::measure_length(from_track, measure_lengths);
        total += Self::calc_duration(1.0 - Self::pos_in_measure(from), from_len, bpm);
        for measure in (from_track + 1)..to_track {
            let len = Self::measure_length(measure, measure_lengths);
            total += Self::calc_duration(1.0, len, bpm);
        }
        let to_len = Self::measure_length(to_track, measure_lengths);
        total += Self::calc_duration(Self::pos_in_measure(to), to_len, bpm);
        total
    }

    fn calc_duration(fraction: f64, section_len: f64, bpm: f64) -> f64 {
        if fraction <= 0.0 || bpm <= 0.0 {
            return 0.0;
        }
        let quarter_note_ms = 60000.0 / bpm;
        let measure_ms = quarter_note_ms * 4.0 * section_len;
        measure_ms * fraction
    }

    fn pos_in_measure(time: ObjTime) -> f64 {
        let denom = time.denominator_u64() as f64;
        if denom == 0.0 {
            return 0.0;
        }
        time.numerator() as f64 / denom
    }

    fn measure_length(measure: u64, measure_lengths: &[f64]) -> f64 {
        measure_lengths
            .get(measure as usize)
            .copied()
            .unwrap_or(1.0)
    }

    fn stop_units_to_ms(units: f64, bpm: f64) -> f64 {
        if units <= 0.0 || bpm <= 0.0 {
            return 0.0;
        }
        let quarter_note_ms = 60000.0 / bpm;
        let whole_note_ms = quarter_note_ms * 4.0;
        whole_note_ms * (units / 192.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_bms_with_bpm_change() -> Bms {
        let mut bms = Bms::default();
        bms.bpm.bpm = Some(Decimal::from(120));
        let prompt = AlwaysUseNewer;
        bms.bpm
            .push_bpm_change(
                BpmChangeObj {
                    time: ObjTime::new(
                        0,
                        1,
                        NonZeroU64::new(2).expect("2 should be a valid NonZeroU64"),
                    ),
                    bpm: Decimal::from(240),
                },
                &prompt,
            )
            .expect("bpm change should be registered");
        bms
    }

    #[test]
    fn timing_engine_handles_bpm_change_within_measure() {
        let bms = make_bms_with_bpm_change();
        let timing = TimingEngine::new(&bms);

        let half = timing.objtime_to_ms(ObjTime::new(
            0,
            1,
            NonZeroU64::new(2).expect("2 should be a valid NonZeroU64"),
        ));
        let end = timing.objtime_to_ms(ObjTime::new(
            0,
            1,
            NonZeroU64::new(1).expect("1 should be a valid NonZeroU64"),
        ));

        assert!((half - 1000.0).abs() < 0.01);
        assert!((end - 1500.0).abs() < 0.01);
    }

    #[test]
    fn timing_engine_applies_stop_after_event_time() {
        let mut bms = Bms::default();
        bms.bpm.bpm = Some(Decimal::from(120));
        bms.stop.push_stop(StopObj {
            time: ObjTime::new(
                0,
                1,
                NonZeroU64::new(2).expect("2 should be a valid NonZeroU64"),
            ),
            duration: Decimal::from(192),
        });

        let timing = TimingEngine::new(&bms);
        let half = timing.objtime_to_ms(ObjTime::new(
            0,
            1,
            NonZeroU64::new(2).expect("2 should be a valid NonZeroU64"),
        ));
        let later = timing.objtime_to_ms(ObjTime::new(
            0,
            3,
            NonZeroU64::new(4).expect("4 should be a valid NonZeroU64"),
        ));

        assert!((half - 1000.0).abs() < 0.01);
        assert!((later - 3500.0).abs() < 0.01);
    }

    #[test]
    fn timing_engine_uses_section_length_per_measure() {
        let mut bms = Bms::default();
        bms.bpm.bpm = Some(Decimal::from(120));
        let prompt = AlwaysUseNewer;
        bms.section_len
            .push_section_len_change(
                SectionLenChangeObj {
                    track: Track(1),
                    length: Decimal::from(0.5),
                },
                &prompt,
            )
            .expect("section length should be registered");

        let timing = TimingEngine::new(&bms);
        let measure_two_start = timing.objtime_to_ms(ObjTime::new(
            2,
            0,
            NonZeroU64::new(1).expect("1 should be a valid NonZeroU64"),
        ));

        assert!((measure_two_start - 3000.0).abs() < 0.01);
    }

    #[test]
    fn timing_engine_applies_bpm_before_stop_at_same_time() {
        let mut bms = Bms::default();
        bms.bpm.bpm = Some(Decimal::from(120));
        let prompt = AlwaysUseNewer;
        let half = ObjTime::new(
            0,
            1,
            NonZeroU64::new(2).expect("2 should be a valid NonZeroU64"),
        );

        bms.bpm
            .push_bpm_change(
                BpmChangeObj {
                    time: half,
                    bpm: Decimal::from(240),
                },
                &prompt,
            )
            .expect("bpm change should be registered");
        bms.stop.push_stop(StopObj {
            time: half,
            duration: Decimal::from(192),
        });

        let timing = TimingEngine::new(&bms);
        let later = timing.objtime_to_ms(ObjTime::new(
            0,
            3,
            NonZeroU64::new(4).expect("4 should be a valid NonZeroU64"),
        ));

        assert!((later - 2250.0).abs() < 0.01);
    }

    #[test]
    fn timing_engine_applies_stp_in_milliseconds() {
        let mut bms = Bms::default();
        bms.bpm.bpm = Some(Decimal::from(120));
        let half = ObjTime::new(
            0,
            1,
            NonZeroU64::new(2).expect("2 should be a valid NonZeroU64"),
        );

        bms.stop.stp_events.insert(
            half,
            StpEvent {
                time: half,
                duration: Duration::from_millis(500),
            },
        );

        let timing = TimingEngine::new(&bms);
        let later = timing.objtime_to_ms(ObjTime::new(
            0,
            3,
            NonZeroU64::new(4).expect("4 should be a valid NonZeroU64"),
        ));

        assert!((later - 2000.0).abs() < 0.01);
    }
}

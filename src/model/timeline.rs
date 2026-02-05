use super::bms_model::{BgaEvent, BgmEvent};
use super::note::Note;

/// A single point in time within the chart, containing notes and events at that time.
#[derive(Debug, Clone)]
pub struct Timeline {
    /// Time in microseconds from song start.
    pub time_us: i64,
    /// Notes at this time, indexed by lane.
    pub notes: Vec<Option<Note>>,
    /// BGM events at this time.
    pub bgm_events: Vec<BgmEvent>,
    /// BGA events at this time.
    pub bga_events: Vec<BgaEvent>,
    /// BPM at this time.
    pub bpm: f64,
    /// Scroll speed multiplier at this time.
    pub scroll: f64,
}

impl Timeline {
    pub fn new(time_us: i64, lane_count: usize) -> Self {
        Self {
            time_us,
            notes: vec![None; lane_count],
            bgm_events: Vec::new(),
            bga_events: Vec::new(),
            bpm: 0.0,
            scroll: 1.0,
        }
    }
}

/// Collection of timelines for the entire chart, sorted by time.
#[derive(Debug, Clone)]
pub struct Timelines {
    /// All timelines sorted by time_us.
    timelines: Vec<Timeline>,
    /// Lane count for this chart.
    lane_count: usize,
}

impl Timelines {
    pub fn new(lane_count: usize) -> Self {
        Self {
            timelines: Vec::new(),
            lane_count,
        }
    }

    /// Add a timeline. Maintains sorted order by time_us.
    pub fn add(&mut self, timeline: Timeline) {
        let pos = self
            .timelines
            .binary_search_by_key(&timeline.time_us, |t| t.time_us)
            .unwrap_or_else(|e| e);
        self.timelines.insert(pos, timeline);
    }

    /// Get or create a timeline at the given time.
    pub fn get_or_create(&mut self, time_us: i64) -> &mut Timeline {
        match self.timelines.binary_search_by_key(&time_us, |t| t.time_us) {
            Ok(idx) => &mut self.timelines[idx],
            Err(idx) => {
                self.timelines
                    .insert(idx, Timeline::new(time_us, self.lane_count));
                &mut self.timelines[idx]
            }
        }
    }

    pub fn timelines(&self) -> &[Timeline] {
        &self.timelines
    }

    pub fn timelines_mut(&mut self) -> &mut [Timeline] {
        &mut self.timelines
    }

    pub fn lane_count(&self) -> usize {
        self.lane_count
    }

    pub fn len(&self) -> usize {
        self.timelines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.timelines.is_empty()
    }

    /// Collect all notes from all timelines, flattened and sorted by time.
    pub fn all_notes(&self) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self
            .timelines
            .iter()
            .flat_map(|tl| tl.notes.iter().filter_map(|n| n.as_ref()))
            .collect();
        notes.sort_by_key(|n| n.time_us);
        notes
    }

    /// Collect notes for a specific lane, sorted by time.
    pub fn lane_notes(&self, lane: usize) -> Vec<&Note> {
        self.timelines
            .iter()
            .filter_map(|tl| tl.notes.get(lane).and_then(|n| n.as_ref()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::note::{NoteType, PlayMode};

    fn make_note(lane: usize, time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::Normal,
            time_us,
            end_time_us: 0,
            wav_id: 0,
            damage: 0.0,
        }
    }

    #[test]
    fn timelines_sorted_insertion() {
        let mut tls = Timelines::new(8);
        tls.add(Timeline::new(2_000_000, 8));
        tls.add(Timeline::new(1_000_000, 8));
        tls.add(Timeline::new(3_000_000, 8));

        let times: Vec<i64> = tls.timelines().iter().map(|t| t.time_us).collect();
        assert_eq!(times, vec![1_000_000, 2_000_000, 3_000_000]);
    }

    #[test]
    fn get_or_create_existing() {
        let mut tls = Timelines::new(8);
        tls.get_or_create(1_000_000);
        tls.get_or_create(1_000_000); // should not duplicate
        assert_eq!(tls.len(), 1);
    }

    #[test]
    fn get_or_create_new() {
        let mut tls = Timelines::new(8);
        tls.get_or_create(1_000_000);
        tls.get_or_create(2_000_000);
        assert_eq!(tls.len(), 2);
    }

    #[test]
    fn all_notes_sorted() {
        let lane_count = PlayMode::Beat7K.lane_count();
        let mut tls = Timelines::new(lane_count);

        let tl = tls.get_or_create(2_000_000);
        tl.notes[0] = Some(make_note(0, 2_000_000));

        let tl = tls.get_or_create(1_000_000);
        tl.notes[1] = Some(make_note(1, 1_000_000));

        let notes = tls.all_notes();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].time_us, 1_000_000);
        assert_eq!(notes[1].time_us, 2_000_000);
    }

    #[test]
    fn lane_notes_filtered() {
        let lane_count = PlayMode::Beat7K.lane_count();
        let mut tls = Timelines::new(lane_count);

        let tl = tls.get_or_create(1_000_000);
        tl.notes[0] = Some(make_note(0, 1_000_000));
        tl.notes[1] = Some(make_note(1, 1_000_000));

        let tl = tls.get_or_create(2_000_000);
        tl.notes[0] = Some(make_note(0, 2_000_000));

        let lane0 = tls.lane_notes(0);
        assert_eq!(lane0.len(), 2);

        let lane1 = tls.lane_notes(1);
        assert_eq!(lane1.len(), 1);
    }
}
